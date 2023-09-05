use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use js_sys::{Uint8Array, JsString, Reflect};
use wasm_bindgen::{JsValue, JsCast};
use wasm_bindgen::prelude::Closure;
use wasm_bindgen_futures::JsFuture;
use wasm_peers::one_to_many::MiniClient;
use wasm_peers::{get_random_session_id, ConnectionType, SessionId};
use web_sys::{EncodedVideoChunkInit, EncodedVideoChunk, VideoDecoder, HtmlImageElement, HtmlCanvasElement, CanvasRenderingContext2d, VideoFrame, VideoDecoderInit, VideoDecoderConfig, EncodedAudioChunkInit, EncodedAudioChunk, MediaStreamTrackGenerator, MediaStreamTrackGeneratorInit, AudioData, AudioDecoder, AudioDecoderInit, AudioDecoderConfig, AudioDataInit};
use yew::{html, Component, Context, Html, NodeRef};
use log::error;

use crate::config::configure_audio_context;
use crate::constants::{VIDEO_CODEC, AUDIO_CODEC, AUDIO_CHANNELS, AUDIO_SAMPLE_RATE};
use crate::crypto::aes::Aes128State;
use crate::inputs::Message;
use crate::utils;
use crate::wrappers::{EncodedVideoChunkTypeWrapper, EncodedAudioChunkTypeWrapper};

pub enum Msg {
    UpdateValue,
}

pub struct Client {
    mini_client: MiniClient,
    host_area: NodeRef,
    client_area: NodeRef,
    is_screen_share: Rc<RefCell<bool>>,
}

const TEXTAREA_ID: &str = "document-textarea";
const TEXTAREA_ID_CLIENT: &str = "client-textarea";

impl Component for Client {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let query_params = utils::get_query_params_multi();
        let session_id =
            match query_params.get("session_id") {
                Some(session_string) => {
                    SessionId::new(session_string)
                }
                _ => {
                    let location = utils::global_window().location();
                    let generated_session_id = get_random_session_id();
                    query_params.append("session_id", generated_session_id.as_str());
                    let search: String = query_params.to_string().into();
                    if let Err(error) = location.set_search(&search) {
                        error!("Error while setting URL: {error:?}")
                    }
                    generated_session_id
                }
            };
        
        let host_area = NodeRef::default();
        let client_area = NodeRef::default();

        let is_ready = Rc::new(RefCell::new(false));
        let connection_type = ConnectionType::StunAndTurn {
            stun_urls: env!("STUN_SERVER_URLS").to_string(),
            turn_urls: env!("TURN_SERVER_URLS").to_string(),
            username: env!("TURN_SERVER_USERNAME").to_string(),
            credential: env!("TURN_SERVER_CREDENTIAL").to_string(),
        };
        let signaling_server_url = concat!(env!("SIGNALING_SERVER_URL"), "/one-to-many");
        let mut mini_client = MiniClient::new(signaling_server_url, session_id.clone(), connection_type)
        .expect("failed to create network manager");

        let on_open_callback = {
            let mini_client = mini_client.clone();
            let is_ready = Rc::clone(&is_ready);
            move || {
                let text_area = match utils::get_text_area(TEXTAREA_ID) {
                    Ok(text_area) => text_area,
                    Err(err) => {
                        log::error!("failed to get textarea: {:#?}", err);
                        return;
                    }
                };
                if !*is_ready.borrow() {
                    text_area.set_disabled(false);
                    text_area.set_placeholder(
                        "This is a live document shared with other users.\nWhat you write will be \
                         visible to everyone.",
                    );
                    *is_ready.borrow_mut() = true;
                }
                let value = text_area.value();
                log::info!("message from value {}", value.clone());
                let message = Message::Init { message: value.clone() };
                let message = serde_json::to_string(&message).unwrap();
                if !value.is_empty() {
                    mini_client
                        .send_message_to_host(&message)
                        .expect("failed to send current input to new connection");
                }
            }
        };

        let is_screen_share = Rc::new(RefCell::new(false));
        let decoder = create_video_decoder("render".to_owned());
        let screen_share_decoder = create_video_decoder("screen_share".to_owned());
        let audio_decoder = Box::new(create_audio_decoder());
        let on_message_callback = {
            let aes = Arc::new(Aes128State::new(true));
            let is_screen_share = is_screen_share.clone();
            let decoder = decoder.clone();
            let screen_share_decoder = screen_share_decoder.clone();
            let audio_decoder = audio_decoder.clone();
            move |message: String| {
                let _ = match serde_json::from_str::<Message>(&message) {
                    Ok(input) => {
                        match input {
                            Message::HostToHost { message } => {
                                log::info!("input {}", message);   
                                match utils::get_text_area(TEXTAREA_ID) {
                                    Ok(text_area) => {
                                        text_area.set_value(&message);
                                    }
                                    Err(err) => {
                                        log::error!("failed to get textarea: {:#?}", err);
                                    }
                                }
                            },
                            Message::HostToClient { message } => {
                                log::info!("input {}", message);   
                                match utils::get_text_area(TEXTAREA_ID_CLIENT) {
                                    Ok(text_area) => {
                                        text_area.set_value(&message);
                                    }
                                    Err(err) => {
                                        log::error!("failed to get textarea: {:#?}", err);
                                    }
                                }
                            },
                            Message::Init { message } => {
                                log::info!("message init {}", message);
                                match utils::get_text_area(TEXTAREA_ID) {
                                    Ok(text_area) => {
                                        text_area.set_value(&message);
                                    }
                                    Err(err) => {
                                        log::error!("failed to get textarea: {:#?}", err);
                                    }
                                }
                            },
                            Message::HostVideo { 
                                message,
                                chunk_type,
                                timestamp
                            } => {
                                let chunk_type = EncodedVideoChunkTypeWrapper::from(chunk_type.as_str()).0;
                                let video_data = Uint8Array::new_with_length(message.len().try_into().unwrap());
                                video_data.copy_from(&message);
                                let video_chunk = EncodedVideoChunkInit::new(&video_data, timestamp, chunk_type);
                                // video_chunk.duration(image.duration);
                                let chunk = EncodedVideoChunk::new(&video_chunk).unwrap();
                                

                                let mut video_vector = vec![0u8; chunk.byte_length() as usize];
                                let video_message = video_vector.as_mut();
                                chunk.copy_to_with_u8_array(video_message);
                                let data = Uint8Array::from(video_message.as_ref());
                                let encoded_video_chunk = EncodedVideoChunk::new(
                                    &EncodedVideoChunkInit::new(&data, chunk.timestamp(), chunk.type_())
                                ).unwrap();
                                match decoder.state() {
                                    web_sys::CodecState::Unconfigured => {
                                        log::info!("video decoder unconfigured");
                                    },
                                    web_sys::CodecState::Configured => {
                                        decoder.decode(&encoded_video_chunk);
                                    },
                                    web_sys::CodecState::Closed => {
                                        log::info!("video decoder closed");
                                    },
                                    _ => {},
                                }
                            },
                            Message::HostScreenShare { 
                                message,
                                chunk_type,
                                timestamp
                            } => {
                                *is_screen_share.borrow_mut() = true;
                                let chunk_type = EncodedVideoChunkTypeWrapper::from(chunk_type.as_str()).0;
                                let video_data = Uint8Array::new_with_length(message.len().try_into().unwrap());
                                video_data.copy_from(&message);
                                let video_chunk = EncodedVideoChunkInit::new(&video_data, timestamp, chunk_type);
                                // video_chunk.duration(image.duration);
                                let chunk = EncodedVideoChunk::new(&video_chunk).unwrap();
                                

                                let mut video_vector = vec![0u8; chunk.byte_length() as usize];
                                let video_message = video_vector.as_mut();
                                chunk.copy_to_with_u8_array(video_message);
                                let data = Uint8Array::from(video_message.as_ref());
                                let encoded_video_chunk = EncodedVideoChunk::new(
                                    &EncodedVideoChunkInit::new(&data, chunk.timestamp(), chunk.type_())
                                ).unwrap();
                                match screen_share_decoder.state() {
                                    web_sys::CodecState::Unconfigured => {
                                        log::info!("video decoder unconfigured");
                                    },
                                    web_sys::CodecState::Configured => {
                                        screen_share_decoder.decode(&encoded_video_chunk);
                                    },
                                    web_sys::CodecState::Closed => {
                                        log::info!("video decoder closed");
                                    },
                                    _ => {},
                                }
                            },
                            Message::HostAudio { 
                                message,
                                chunk_type,
                                timestamp,
                                duration
                            } => {     
                   
                                let chunk_type = EncodedAudioChunkTypeWrapper::from(chunk_type).0;
                                // log::info!("audio {:?}", message);
                                let audio_data = &message;
                                let audio_data_js: js_sys::Uint8Array =
                                    js_sys::Uint8Array::new_with_length(audio_data.len() as u32);
                                audio_data_js.copy_from(audio_data.as_slice());
                                let chunk_type = EncodedAudioChunkTypeWrapper(chunk_type);
                                let mut audio_chunk_init =
                                    EncodedAudioChunkInit::new(&audio_data_js.into(), timestamp, chunk_type.0);
                                audio_chunk_init.duration(duration);
                                let encoded_audio_chunk = EncodedAudioChunk::new(&audio_chunk_init).unwrap();

                                match audio_decoder.state() {
                                    web_sys::CodecState::Unconfigured => {
                                        log::info!("audio decoder unconfigured");
                                    },
                                    web_sys::CodecState::Configured => {
                                        log::info!("configured");
                                        // audio_decoder.decode(&encoded_audio_chunk);
                                    },
                                    web_sys::CodecState::Closed => {
                                        log::info!("audio_decoder closed");
                                    },
                                    _ => {}
                                }
                                       
                                                
                            },
                        }
                    },
                    Err(err) => {
                        log::error!("failed to get input message: {:#?}", err);
                    },
                };
            } 
        
        };
        
        mini_client.start(on_open_callback, on_message_callback);
        Self {
            mini_client,
            host_area,
            client_area,
            is_screen_share,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Self::Message::UpdateValue => match utils::get_text_area_from_noderef(&self.client_area) {
                Ok(text_area) => {
                    let _ = self.mini_client.send_message_to_host(&text_area.value());
                    true
                }
                Err(err) => {
                    log::error!("failed to get textarea: {:#?}", err);
                    false
                }
            },
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let oninput = ctx.link().callback(|_| Self::Message::UpdateValue);
        let disabled = true;
        let placeholder = "This is a live document shared with other users.\nYou will be allowed \
                           to write once other join, or your connection is established.";
        let is_screen = self.is_screen_share.borrow();
        log::info!("self is screen {}", self.is_screen_share.borrow().clone());
        
        html! {
            <main class="px-3">
                if !*is_screen {
                    <div class="row">
                        <div class="col-6">
                            <textarea id={ TEXTAREA_ID_CLIENT } ref={ self.client_area.clone() } class="document" cols="100" rows="30" { placeholder } { oninput }/>
                        </div>
                        <div class="col-6">
                            <textarea id={ TEXTAREA_ID } ref={ self.host_area.clone() } class="document" cols="100" rows="30" { disabled } { placeholder } />
                        </div>
                    </div>
                    <div class="consumer">
                        <h3>{"Consumer!"}</h3>
                        <canvas id="render" class="client_canvas" ></canvas>
                    </div>
                    <div class="consumer">
                        <h3>{"демонстрация экрана"}</h3>
                        <canvas id="screen_share" class="client_canvas" ></canvas>
                    </div>
                } else {
                    <div class="consumer">
                        <canvas id="screen_share" class="screen_canvas" ></canvas>
                    </div>
                }
                
            </main>
        }
    }
}

fn create_video_decoder(render_id: String) -> VideoDecoder {
    let error_video = Closure::wrap(Box::new(move |e: JsValue| {
    }) as Box<dyn FnMut(JsValue)>);

    let output = Closure::wrap(Box::new(move |original_chunk: JsValue| {
        let chunk = Box::new(original_chunk);
        let video_chunk = chunk.clone().unchecked_into::<HtmlImageElement>();
        let width = Reflect::get(&chunk.clone(), &JsString::from("codedWidth"))
            .unwrap()
            .as_f64()
            .unwrap();
        let height = Reflect::get(&chunk.clone(), &JsString::from("codedHeight"))
            .unwrap()
            .as_f64()
            .unwrap();

        let render_canvas = utils::get_window()
            .unwrap()
            .document()
            .unwrap()
            .get_element_by_id(&render_id)
            .unwrap()
            .unchecked_into::<HtmlCanvasElement>();

        render_canvas.set_width(width as u32);
        render_canvas.set_height(height as u32);

        let ctx = render_canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .unchecked_into::<CanvasRenderingContext2d>();

        let _ = ctx.draw_image_with_html_image_element(
            &video_chunk, 
            0.0 as f64,
            0.0 as f64
        );

        video_chunk.unchecked_into::<VideoFrame>().close();
    }) as Box<dyn FnMut(JsValue)>);

    let local_video_decoder = VideoDecoder::new(
        &VideoDecoderInit::new(error_video.as_ref().unchecked_ref(), output.as_ref().unchecked_ref())
    ).unwrap();
    error_video.forget();
    output.forget();
    local_video_decoder.configure(&VideoDecoderConfig::new(&VIDEO_CODEC));
    local_video_decoder
}

fn create_audio_decoder() -> AudioDecoder {
    let error = Closure::wrap(Box::new(move |e: JsValue| {
        error!("{:?}", e);
    }) as Box<dyn FnMut(JsValue)>);
    let audio_stream_generator =
        MediaStreamTrackGenerator::new(&MediaStreamTrackGeneratorInit::new("audio")).unwrap();
    // The audio context is used to reproduce audio.
    let audio_context = configure_audio_context(&audio_stream_generator).unwrap();

    let output = Closure::wrap(Box::new(move |audio_data: AudioData| {
        let writable = audio_stream_generator.writable();
        if writable.locked() {
            return;
        }
        if let Err(e) = writable.get_writer().map(|writer| {
            wasm_bindgen_futures::spawn_local(async move {
                if let Err(e) = JsFuture::from(writer.ready()).await {
                    error!("write chunk error {:?}", e);
                }
                if let Err(e) = JsFuture::from(writer.write_with_chunk(&audio_data)).await {
                    error!("write chunk error {:?}", e);
                };
                writer.release_lock();
            });
        }) {
            error!("error {:?}", e);
        }
    }) as Box<dyn FnMut(AudioData)>);
    let decoder = AudioDecoder::new(&AudioDecoderInit::new(
        error.as_ref().unchecked_ref(),
        output.as_ref().unchecked_ref(),
    ))
    .unwrap();
    decoder.configure(&AudioDecoderConfig::new(
        AUDIO_CODEC,
        AUDIO_CHANNELS,
        AUDIO_SAMPLE_RATE,
    ));
    // let decoder = AudioDecoder::new(&audio_context).unwrap();
    decoder
}