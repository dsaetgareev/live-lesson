use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use gloo_timers::callback::Timeout;
use js_sys::Uint8Array;
use wasm_peers::one_to_many::MiniClient;
use wasm_peers::{get_random_session_id, ConnectionType, SessionId};
use web_sys::{EncodedVideoChunkInit, EncodedVideoChunk, EncodedAudioChunkInit, EncodedAudioChunk};
use yew::{html, Component, Context, Html, NodeRef};
use log::error;

use crate::encoders::camera_encoder::CameraEncoder;
use crate::crypto::aes::Aes128State;
use crate::utils::device::{create_video_decoder, create_audio_decoder};
use crate::utils::inputs::Message;
use crate::utils::inputs::ClientMessage;
use crate::utils;
use crate::wrappers::{EncodedVideoChunkTypeWrapper, EncodedAudioChunkTypeWrapper};
use crate::media_devices::device_selector::DeviceSelector;


const TEXTAREA_ID: &str = "document-textarea";
const TEXTAREA_ID_CLIENT: &str = "client-textarea";
const VIDEO_ELEMENT_ID: &str = "webcam";

pub enum Msg {
    UpdateValue,
    VideoDeviceChanged(String),
    EnableVideo(bool),
    AudioDeviceChanged(String),
}

pub struct Client {
    mini_client: MiniClient,
    host_area: NodeRef,
    client_area: NodeRef,
    is_screen_share: Rc<RefCell<bool>>,
    camera: CameraEncoder,
}

impl Component for Client {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let query_params = utils::dom::get_query_params_multi();
        let session_id =
            match query_params.get("session_id") {
                Some(session_string) => {
                    SessionId::new(session_string)
                }
                _ => {
                    let location = utils::dom::global_window().location();
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
                let text_area = match utils::dom::get_text_area(TEXTAREA_ID) {
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
                                match utils::dom::get_text_area(TEXTAREA_ID) {
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
                                match utils::dom::get_text_area(TEXTAREA_ID_CLIENT) {
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
                                match utils::dom::get_text_area(TEXTAREA_ID) {
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
                                timestamp,
                                duration
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
                                let mut encoded_chunk_init = EncodedVideoChunkInit::new(&data, chunk.timestamp(), chunk.type_());
                                encoded_chunk_init.duration(duration);
                                let encoded_video_chunk = EncodedVideoChunk::new(
                                    &encoded_chunk_init
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
                                timestamp,
                                duration,
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
                                let mut encoded_chunk_init = EncodedVideoChunkInit::new(&data, chunk.timestamp(), chunk.type_());
                                encoded_chunk_init.duration(duration);
                                let encoded_video_chunk = EncodedVideoChunk::new(
                                    &encoded_chunk_init
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
        let aes = Arc::new(Aes128State::new(true));
        Self {
            mini_client,
            host_area,
            client_area,
            is_screen_share,
            camera: CameraEncoder::new(aes.clone()),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::UpdateValue => match utils::dom::get_text_area_from_noderef(&self.client_area) {
                Ok(text_area) => {
                    let message = ClientMessage::ClientText { message: text_area.value() };
                    let message = serde_json::to_string(&message).unwrap();
                    let _ = self.mini_client.send_message_to_host(&message);
                    true
                }
                Err(err) => {
                    log::error!("failed to get textarea: {:#?}", err);
                    false
                }
            },
            Msg::VideoDeviceChanged(video) => {
                if self.camera.select(video) {
                    log::info!("selected");
                    let link = ctx.link().clone();
                    let timeout = Timeout::new(1000, move || {
                        link.send_message(Msg::EnableVideo(true));
                    });
                    timeout.forget();
                }
                false
            }
            Msg::EnableVideo(should_enable) => {
                if !should_enable {
                    return true;
                }

                let ms = self.mini_client.clone();
                let on_frame = move |chunk: web_sys::EncodedVideoChunk| {
                    let duration = chunk.duration().expect("no duration video chunk");
                    let mut buffer: [u8; 100000] = [0; 100000];
                    let byte_length = chunk.byte_length() as usize;
                    chunk.copy_to_with_u8_array(&mut buffer);
                    let data = buffer[0..byte_length].to_vec();
                    let chunk_type = EncodedVideoChunkTypeWrapper(chunk.type_()).to_string();
                    let timestamp = chunk.timestamp();
                    // let data = aes.encrypt(&data).unwrap();
                    
                    let message = ClientMessage::ClientVideo { 
                        message: data,
                        chunk_type,
                        timestamp,
                        duration
                    };
                    match serde_json::to_string(&message) {
                        Ok(message) => {
                            let _ = ms.send_message_to_host(&message);
                        },
                        Err(_) => todo!(),
                    };
                    
                };
                self.camera.start(
                    "email".to_owned(),
                    on_frame,
                    VIDEO_ELEMENT_ID,
                );
                log::info!("camera started");
                false
            },
            Msg::AudioDeviceChanged(audio) => {
                false
            },
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let oninput = ctx.link().callback(|_| Self::Message::UpdateValue);
        let disabled = true;
        let placeholder = "This is a live document shared with other users.\nYou will be allowed \
                           to write once other join, or your connection is established.";
        let is_screen = self.is_screen_share.borrow();

        let mic_callback = ctx.link().callback(Msg::AudioDeviceChanged);
        let cam_callback = ctx.link().callback(Msg::VideoDeviceChanged);
        
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
                    <DeviceSelector on_microphone_select={mic_callback} on_camera_select={cam_callback}/>
                    <div class="consumer">
                        <h3>{"Consumer!"}</h3>
                        <video class="self-camera" autoplay=true id={VIDEO_ELEMENT_ID}></video>
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