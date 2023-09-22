use std::cell::RefCell;
use std::rc::Rc;
use std::str::FromStr;
use std::sync::Arc;
use gloo_timers::callback::Timeout;
use wasm_bindgen::JsCast;
use wasm_peers::one_to_many::MiniClient;
use wasm_peers::{get_random_session_id, ConnectionType, SessionId};
use web_sys::{EncodedAudioChunkInit, EncodedAudioChunk, InputEvent, HtmlTextAreaElement};
use yew::{html, Component, Context, Html, NodeRef};
use log::error;

use crate::encoders::camera_encoder::CameraEncoder;
use crate::crypto::aes::Aes128State;
use crate::encoders::microphone_encoder::MicrophoneEncoder;
use crate::models::client::ClientProps;
use crate::models::host::HostPorps;
use crate::models::packet::VideoPacket;
use crate::utils::device::{create_video_decoder, create_audio_decoder, create_video_decoder_frame};
use crate::utils::inputs::Message;
use crate::utils::inputs::ClientMessage;
use crate::utils;
use crate::wrappers::EncodedAudioChunkTypeWrapper;
use crate::media_devices::device_selector::DeviceSelector;

use super::client_manager::ClientManager;


const TEXTAREA_ID: &str = "document-textarea";
const TEXTAREA_ID_CLIENT: &str = "client-textarea";
const VIDEO_ELEMENT_ID: &str = "webcam";

pub enum Msg {
    Init,
    Tick,
    UpdateValue(String),
    VideoDeviceChanged(String),
    EnableVideo(bool),
    AudioDeviceChanged(String),
    EnableMicrophone(bool),
    SwitchVedeo,
}

pub struct Client {
    session_id: SessionId,
    client_manager: Option<ClientManager>,
    host_props: Rc<RefCell<HostPorps>>,
    client_props: Rc<RefCell<ClientProps>>,
    camera: CameraEncoder,
    microphone: MicrophoneEncoder,
}

impl Component for Client {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
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
        ctx.link().send_message(Msg::Init);
        Self {
            session_id,
            client_manager: None,
            host_props: Rc::new(RefCell::new(HostPorps::new())),
            client_props: Rc::new(RefCell::new(ClientProps::new(String::default(), String::default()))),
            camera: CameraEncoder::new(),
            microphone: MicrophoneEncoder::new(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Init => {
                let link = ctx.link().clone();
                let on_tick = move || {
                    link.send_message(Msg::Tick)
                };
                self.client_manager = Some(init(
                    self.session_id.clone(),
                    on_tick,
                    self.host_props.clone(),
                    self.client_props.clone(),
                ));
                true
            },
            Msg::Tick => {
                true
            }
            Msg::UpdateValue(content) => {
                self.client_props.borrow_mut().client_content = content.clone();
                let message = ClientMessage::ClientText { message: content };
                let message = serde_json::to_string(&message).unwrap();
                let _ = self.client_manager.as_mut().unwrap().mini_client.send_message_to_host(&message);
                true
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

                let ms = self.client_manager.as_mut().unwrap().mini_client.clone();
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

                let ms = self.client_manager.as_mut().unwrap().mini_client.clone();
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
        let oninput = ctx.link().callback(|e:InputEvent| {
            let content = e
                .target()
                .unwrap()
                .unchecked_into::<HtmlTextAreaElement>()
                .value();
            Self::Message::UpdateValue(content)
        });
        let disabled = false;
        let placeholder = "This is a live document shared with other users.\nYou will be allowed \
                           to write once other join, or your connection is established.";

        let mic_callback = ctx.link().callback(Msg::AudioDeviceChanged);
        let cam_callback = ctx.link().callback(Msg::VideoDeviceChanged);
        let on_video_btn = ctx.link().callback(|_| Msg::SwitchVedeo);
        let host_value = self.host_props.borrow().host_area_content.content.clone();
        let client_value = self.client_props.borrow().client_content.clone();
        html! {
            <main class="px-3">
                <div id="container">
                    <div class="row">
                        <div class="col-6">
                            <textarea id={ TEXTAREA_ID_CLIENT } value={ client_value } class="document" cols="100" rows="30" { placeholder } { oninput }/>
                        </div>
                        <div class="col-6">
                            <textarea id={ TEXTAREA_ID } value={ host_value } class="document" cols="100" rows="30" { disabled } { placeholder } />
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

fn init(
    session_id: SessionId,
    on_tick: impl Fn() + 'static,
    host_content: Rc<RefCell<HostPorps>>,
    client_props: Rc<RefCell<ClientProps>>,
) -> ClientManager {

    let mut host_manager = ClientManager::new(session_id);
    host_manager.init(on_tick, host_content, client_props);
    host_manager
}