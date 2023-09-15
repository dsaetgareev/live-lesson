use std::cell::RefCell;
use std::rc::Rc;
use std::str::FromStr;
use std::sync::Arc;
use gloo_timers::callback::Timeout;
use wasm_peers::one_to_many::MiniClient;
use wasm_peers::{get_random_session_id, ConnectionType, SessionId};
use web_sys::{EncodedAudioChunkInit, EncodedAudioChunk};
use yew::{html, Component, Context, Html, NodeRef};
use log::error;

use crate::encoders::camera_encoder::CameraEncoder;
use crate::crypto::aes::Aes128State;
use crate::encoders::microphone_encoder::MicrophoneEncoder;
use crate::models::packet::VideoPacket;
use crate::utils::device::{create_video_decoder, create_audio_decoder, create_video_decoder_frame};
use crate::utils::inputs::Message;
use crate::utils::inputs::ClientMessage;
use crate::utils;
use crate::wrappers::EncodedAudioChunkTypeWrapper;
use crate::media_devices::device_selector::DeviceSelector;


const TEXTAREA_ID: &str = "document-textarea";
const TEXTAREA_ID_CLIENT: &str = "client-textarea";
const VIDEO_ELEMENT_ID: &str = "webcam";

pub enum Msg {
    UpdateValue,
    VideoDeviceChanged(String),
    EnableVideo(bool),
    AudioDeviceChanged(String),
    EnableMicrophone(bool),
    SwitchVedeo,
}

pub struct Client {
    mini_client: MiniClient,
    host_area: NodeRef,
    client_area: NodeRef,
    camera: CameraEncoder,
    microphone: MicrophoneEncoder,
}

impl Component for Client {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let query_params = utils::dom::get_query_params_multi();
        let session_id =
            match query_params.get("session_id") {
                Some(session_string) => {
                    SessionId::new(uuid::Uuid::from_str(&session_string).unwrap().as_u128())
                }
                _ => {
                    let location = utils::dom::global_window().location();
                    let generated_session_id = get_random_session_id();
                    query_params.append("session_id", &generated_session_id.to_string());
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

        let video = create_video_decoder("render".to_owned());
        let screen_share_decoder = create_video_decoder_frame("screen_share".to_owned());
        let audio = create_audio_decoder();
        let on_message_callback = {
            let _aes = Arc::new(Aes128State::new(true));
            let mut video = video.clone();
            let mut screen_share_decoder = screen_share_decoder.clone();
            let mut audio = audio.clone();
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
                            } => {
                                if video.on_video {
                                     if video.check_key {
                                        if message.chunk_type != "key" {
                                            return;
                                        }
                                        video.check_key = false;
                                    }
                                    match video.video_decoder.state() {
                                        web_sys::CodecState::Unconfigured => {
                                            log::info!("video decoder unconfigured");
                                        },
                                        web_sys::CodecState::Configured => {
                                            if let Err(err) = video.decode(Arc::new(message)) {
                                                error!("error on decode {}", err);
                                            }
                                        },
                                        web_sys::CodecState::Closed => {
                                            log::info!("video decoder closed");
                                            video.video_decoder.configure(&video.video_config);
                                            video.check_key = true;
                                        },
                                        _ => {},
                                    }
                                }
                            },
                            Message::HostIsScreenShare { 
                                message
                            } => {
                                on_screen_share(message);
                            },
                            Message::HostScreenShare { 
                                message
                            } => {
                                if screen_share_decoder.check_key {
                                        if message.chunk_type != "key" {
                                            return;
                                        }
                                        screen_share_decoder.check_key = false;
                                } 
                                match screen_share_decoder.video_decoder.state() {
                                    web_sys::CodecState::Unconfigured => {
                                        log::info!("video decoder unconfigured");
                                    },
                                    web_sys::CodecState::Configured => {
                                        let _ = screen_share_decoder.decode_break(Arc::new(message));
                                    },
                                    web_sys::CodecState::Closed => {
                                        log::info!("video decoder closed");
                                        video.video_decoder.configure(&video.video_config);
                                        video.check_key = true;
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
                                if audio.on_speakers {
                                    let _ = audio.audio_context.resume();
                                    let chunk_type = EncodedAudioChunkTypeWrapper::from(chunk_type).0;
                                    let audio_data = &message;
                                    let audio_data_js: js_sys::Uint8Array =
                                        js_sys::Uint8Array::new_with_length(audio_data.len() as u32);
                                    audio_data_js.copy_from(audio_data.as_slice());
                                    let chunk_type = EncodedAudioChunkTypeWrapper(chunk_type);
                                    let mut audio_chunk_init =
                                        EncodedAudioChunkInit::new(&audio_data_js.into(), timestamp, chunk_type.0);
                                    audio_chunk_init.duration(duration);
                                    let encoded_audio_chunk = EncodedAudioChunk::new(&audio_chunk_init).unwrap();
    
                                    match audio.audio_decoder.state() {
                                        web_sys::CodecState::Unconfigured => {
                                            log::info!("audio decoder unconfigured");
                                        },
                                        web_sys::CodecState::Configured => {
                                            audio.audio_decoder.decode(&encoded_audio_chunk);
                                        },
                                        web_sys::CodecState::Closed => {
                                            log::info!("audio_decoder closed");
                                        },
                                        _ => {}
                                    }    
                                }
                                
                            },
                            Message::HostSwicthAudio => {
                                audio.on_speakers = !audio.on_speakers;
                            },
                            Message::HostSwicthVideo => {
                                video.video_start = !video.video_start;
                                if video.video_start {
                                    video.video_decoder.configure(&video.video_config);
                                    video.on_video = !video.on_video;
                                    video.check_key = true;
                                } else {
                                    video.on_video = !video.on_video;
                                    video.video_decoder.reset();
                                }
                            }
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
            camera: CameraEncoder::new(),
            microphone: MicrophoneEncoder::new(),
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
                    let on_video = self.camera.get_enabled();
                    let timeout = Timeout::new(1000, move || {
                        link.send_message(Msg::EnableVideo(on_video));
                    });
                    timeout.forget();
                }
                false
            }
            Msg::SwitchVedeo => {
                let link = ctx.link().clone();
                let on_video = self.camera.get_enabled();
                let on_video = self.camera.set_enabled(!on_video);
                log::info!("{}", on_video);
                if self.camera.get_enabled() {
                    let timeout = Timeout::new(1000, move || {
                        link.send_message(Msg::EnableVideo(on_video));
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
                let on_frame = move |packet: VideoPacket| {
                                       
                    let message = ClientMessage::ClientVideo { 
                        message: packet
                    };
                    match serde_json::to_string(&message) {
                        Ok(message) => {
                            let _ = ms.send_message_to_host(&message);
                        },
                        Err(_) => todo!(),
                    };
                    
                };
                self.camera.start(
                    on_frame,
                    VIDEO_ELEMENT_ID,
                );
                log::info!("camera started");
                false
            },
            Msg::AudioDeviceChanged(audio) => {
                if self.microphone.select(audio) {
                    let link = ctx.link().clone();
                    let timeout = Timeout::new(1000, move || {
                        link.send_message(Msg::EnableMicrophone(true));
                    });
                    timeout.forget();
                }
                false
            },
            Msg::EnableMicrophone(should_enable) => {
                if !should_enable {
                    return true;
                }

                let ms = self.mini_client.clone();
                let on_audio = move |chunk: web_sys::EncodedAudioChunk| {
                    let duration = chunk.duration().unwrap();
                    let mut buffer: [u8; 100000] = [0; 100000];
                    let byte_length = chunk.byte_length() as usize;

                    chunk.copy_to_with_u8_array(&mut buffer);

                    let data = buffer[0..byte_length as usize].to_vec();

                    let chunk_type = EncodedAudioChunkTypeWrapper(chunk.type_()).to_string();
                    let timestamp = chunk.timestamp();
                    // let timestamp = Date::new_0().get_time() as f64;
                    // let data = aes.encrypt(&data).unwrap();
                    let message = ClientMessage::ClientAudio { 
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
                self.microphone.start(
                    on_audio
                );
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let oninput = ctx.link().callback(|_| Self::Message::UpdateValue);
        let disabled = true;
        let placeholder = "This is a live document shared with other users.\nYou will be allowed \
                           to write once other join, or your connection is established.";

        let mic_callback = ctx.link().callback(Msg::AudioDeviceChanged);
        let cam_callback = ctx.link().callback(Msg::VideoDeviceChanged);
        let on_video_btn = ctx.link().callback(|_| Msg::SwitchVedeo);
        
        html! {
            <main class="px-3">
                <div id="container">
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
                        <div>
                            <button onclick={ on_video_btn }>{"Video"}</button>
                        </div>
                        <video class="self-camera" autoplay=true id={VIDEO_ELEMENT_ID}></video>
                        <canvas id="render" class="client_canvas" ></canvas>
                    </div>
                </div>
                
                <div id="shcreen_container" class="consumer unvis">
                    <canvas id="screen_share" class="screen_canvas" ></canvas>
                </div>
            </main>
        }
    }
}

fn on_screen_share(is_share: bool) {
    let common_container = utils::dom::get_element("container").unwrap();
    let shcreen_container = utils::dom::get_element("shcreen_container").unwrap();
    if is_share {
        common_container.set_class_name("unvis");
        shcreen_container.set_class_name("vis");
    } else {
        common_container.set_class_name("vis");
        shcreen_container.set_class_name("unvis");
    }
   
}