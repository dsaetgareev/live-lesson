
use std::sync::Arc;

use gloo_timers::callback::Timeout;
use js_sys::Date;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::Closure;
use wasm_peers::{get_random_session_id, ConnectionType, SessionId, UserId};
use web_sys::{HtmlElement, MouseEvent};
use yew::{html, Component, Context, Html, NodeRef};
use log::error;

use crate::camera_encoder::CameraEncoder;
use crate::crypto::aes::Aes128State;
use crate::device_selector::DeviceSelector;
use crate::host_manager::HostManager;
use crate::inputs::Message;
use crate::microphone_encoder::MicrophoneEncoder;
use crate::screen_encoder::ScreenEncoder;
use crate::utils::{self, get_window};
use crate::wrappers::{EncodedVideoChunkTypeWrapper, EncodedAudioChunkTypeWrapper};

const TEXTAREA_ID: &str = "document-textarea";
const TEXTAREA_ID_CLIENT: &str = "client-textarea";
const VIDEO_ELEMENT_ID: &str = "webcam";

pub enum Msg {
    Init,
    UpdateValue,
    Tick,
    ChooseItem(String),
    SendClient,
    EnableVideo(bool),
    EnableMicrophone(bool),
    EnableScreenShare(bool),
    AudioDeviceChanged(String),
    VideoDeviceChanged(String),
}

pub struct Host {
    session_id: SessionId,
    host_manager: Option<HostManager>,
    tick_callback: Closure<dyn FnMut()>,
    host_area: NodeRef,
    client_area: NodeRef,
    camera: CameraEncoder,
    microphone: MicrophoneEncoder,
    pub screen: ScreenEncoder,
    aes: Arc<Aes128State>,
    
}

impl Component for Host {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let query_params = utils::get_query_params_multi();
        let (session_id, _is_host) =
            match (query_params.get("session_id"), query_params.get("is_host")) {
                (Some(session_string), Some(is_host)) => {
                    (SessionId::new(session_string), is_host == "true")
                }
                _ => {
                    let location = utils::global_window().location();
                    let generated_session_id = get_random_session_id();
                    query_params.append("session_id", generated_session_id.as_str());
                    // query_params.append("host", "true");
                    let search: String = query_params.to_string().into();
                    if let Err(error) = location.set_search(&search) {
                        error!("Error while setting URL: {error:?}")
                    }
                    (generated_session_id, true)
                }
            };

        let tick_callback = {
            let link = ctx.link().clone();
            Closure::wrap(Box::new(move || link.send_message(Msg::Tick)) as Box<dyn FnMut()>)
        };
        let host_area = NodeRef::default();
        let client_area = NodeRef::default();
        ctx.link().send_message(Msg::Init);
        let aes = Arc::new(Aes128State::new(true));
        Self {
            session_id,
            host_manager: None,
            tick_callback,
            host_area,
            client_area,
            camera: CameraEncoder::new(aes.clone()),
            microphone: MicrophoneEncoder::new(aes.clone()),
            screen: ScreenEncoder::new(aes.clone()),
            aes
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Self::Message::Init => {
                
                self.host_manager = Some(init(
                    self.session_id.clone()
                ));
                ctx.link().send_message(Msg::Tick);
                // ctx.link().send_message(Msg::EnableMicrophone(true));
                false
            },
            Self::Message::UpdateValue => match utils::get_text_area_from_noderef(&self.host_area) {
                Ok(text_area) => {
                    let message = Message::HostToHost {
                         message: text_area.value()
                    };
                    let message = serde_json::to_string(&message).unwrap();
                    self.host_manager.as_ref().unwrap().mini_server.send_message_to_all(&message);
                    let lenghtn = self.host_manager.as_ref().unwrap().players.borrow().len();
                    log::info!("players len {}", lenghtn);
                    true
                }
                Err(err) => {
                    log::error!("failed to get textarea: {:#?}", err);
                    false
                }
            },
            Self::Message::Tick => {
                let time = Date::new_0().get_milliseconds() as f64;
                log::info!("ticking {}", time);
                if let Err(error) = get_window().unwrap().request_animation_frame(
                    self.tick_callback.as_ref().unchecked_ref(),
                ) {
                    error!("Failed requesting next animation frame: {error:?}");
                }
                true
            },
            Self::Message::ChooseItem(client_id) => {

                match utils::get_text_area_from_noderef(&self.client_area) {
                    Ok(text_area) => {
                        let _ = text_area.set_attribute("client_id", &client_id).unwrap();
                        let value = self.host_manager
                            .as_ref()
                            .unwrap()
                            .players
                            .borrow()
                            .get(&UserId::new(client_id.parse::<usize>().unwrap()))
                            .unwrap()
                            .clone();
                        text_area.set_value(&value);
                        true
                    }
                    Err(err) => {
                        log::error!("failed to get textarea: {:#?}", err);
                        false
                    }
                }
            },
            Self::Message::SendClient => {

                match utils::get_text_area_from_noderef(&self.client_area) {
                    Ok(text_area) => {
                        let is_client_id = match text_area.get_attribute("client_id") {
                            Some(client_id) => {
                                if client_id != "none".to_owned() {
                                    let user_id: UserId = UserId::new(client_id.parse::<usize>().unwrap());
                                    let value = text_area.value();
                                    let message = Message::HostToClient {
                                        message: value
                                    };
                                    let message = serde_json::to_string(&message).unwrap();
                                    let _ = self.host_manager.as_ref().unwrap().mini_server.send_message(user_id, &message);
                                    return true;
                                }
                                false
                                
                            },
                            None => false,
                        };
                        is_client_id
                    }
                    Err(err) => {
                        log::error!("failed to get textarea: {:#?}", err);
                        false
                    }
                }
            },
            Self::Message::EnableVideo(should_enable) => {
                if !should_enable {
                    return true;
                }

                let aes = self.aes.clone();
                let ms = self.host_manager.as_ref().unwrap().mini_server.clone();
                let on_frame = move |chunk: web_sys::EncodedVideoChunk| {
                    let duration = chunk.duration().expect("no duration video chunk");
                    log::info!("durateion {:?}", duration);
                    let mut buffer: [u8; 100000] = [0; 100000];
                    let byte_length = chunk.byte_length() as usize;
                    chunk.copy_to_with_u8_array(&mut buffer);
                    let data = buffer[0..byte_length].to_vec();
                    let chunk_type = EncodedVideoChunkTypeWrapper(chunk.type_()).to_string();
                    let timestamp = chunk.timestamp();
                    // let data = aes.encrypt(&data).unwrap();
                    let message = Message::HostVideo { 
                        message: data,
                        chunk_type,
                        timestamp,
                    };
                    match serde_json::to_string(&message) {
                        Ok(message) => {
                            ms.send_message_to_all(&message);
                        },
                        Err(_) => todo!(),
                    };
                    // log::info!("data {:?}", data);
                    
                };
                self.camera.start(
                    "email".to_owned(),
                    on_frame,
                    VIDEO_ELEMENT_ID,
                );
                log::info!("camera started");
                false
            },
            Self::Message::EnableScreenShare(should_enable) => {
                if !should_enable {
                    return true;
                }

                let aes = self.aes.clone();
                let ms = self.host_manager.as_ref().unwrap().mini_server.clone();
                let on_frame = move |chunk: web_sys::EncodedVideoChunk| {
                    let mut buffer: [u8; 100000] = [0; 100000];
                    let byte_length = chunk.byte_length() as usize;
                    chunk.copy_to_with_u8_array(&mut buffer);
                    let data = buffer[0..byte_length].to_vec();
                    let chunk_type = EncodedVideoChunkTypeWrapper(chunk.type_()).to_string();
                    let timestamp = chunk.timestamp();
                    // let data = aes.encrypt(&data).unwrap();
                    log::info!("data {:?}", data);
                    let message = Message::HostScreenShare { 
                        message: data,
                        chunk_type,
                        timestamp,
                    };
                    match serde_json::to_string(&message) {
                        Ok(message) => {
                            ms.send_message_to_all(&message);
                        },
                        Err(_) => todo!(),
                    };                    
                };
                self.screen.start(
                    "email".to_owned(),
                    on_frame,
                );
                log::info!("camera started");
                false
            },
            Self::Message::EnableMicrophone(should_enable) => {
                log::info!("im microphone");
                if !should_enable {
                    return true;
                }

                let aes = self.aes.clone();
                let ms = self.host_manager.as_ref().unwrap().mini_server.clone();
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
                    let message = Message::HostAudio { 
                        message: data,
                        chunk_type,
                        timestamp,
                        duration
                    };
                    match serde_json::to_string(&message) {
                        Ok(message) => {
                            ms.send_message_to_all(&message);
                        },
                        Err(_) => todo!(),
                    };                    
                };
                self.microphone.start(
                    "email".to_owned(),
                    on_audio
                );
                log::info!("microphone started");
                false
            }
            Msg::AudioDeviceChanged(audio) => {
                if self.microphone.select(audio) {
                    let link = ctx.link().clone();
                    let timeout = Timeout::new(1000, move || {
                        link.send_message(Msg::EnableMicrophone(false));
                    });
                    timeout.forget();
                }
                false
            }
            Msg::VideoDeviceChanged(video) => {
                log::info!("video {}", video.clone());
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
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {

        let item_click = ctx.link().callback(|e: MouseEvent| {
            let target: HtmlElement = e
                .target()
                .unwrap()
                .dyn_into()
                .unwrap();
            let client_id = target.get_attribute("client_id").unwrap();
            log::info!("client_id {}", client_id.clone());
            
            Msg::ChooseItem(client_id)
        });

        let render_item = |key: String, value: String| {
            let client_id = key.clone();
            html! {
                    <>
                        <div client_id={ client_id.clone() } class="col" onclick={ item_click.clone() }>
                            <textarea id={ key } client_id={ client_id } value={ value } class="doc-item" cols="100" rows="30" />
                        </div>
                    </>
            }
        };

        let render = || {
            match self.host_manager.as_ref() {
                Some(host_manager) => {
                    host_manager.players.borrow().clone()
                        .into_keys()
                        .map(|key| {
                            let value = String::from(host_manager.players.borrow().get(&key).unwrap());
                            log::info!("value {}", value.clone());
                            render_item(key.to_string(), value.to_string())
                        }).collect::<Html>()      
                },
                None => {
                    html!(
                        <div>
                            {"hello"}
                        </div>
                    )                
                },
            }
        };

        let oninput = ctx.link().callback(|_| Self::Message::UpdateValue);
        let placeholder = "This is a live document shared with other users.\nYou will be allowed \
                           to write once other join, or your connection is established.";
        let client_id = "none";
        let oninput_client = ctx.link().callback(|_| Self::Message::SendClient);

        let mic_callback = ctx.link().callback(Msg::AudioDeviceChanged);
        let cam_callback = ctx.link().callback(Msg::VideoDeviceChanged);

        let screen_share_cb = ctx.link().callback(|_| Msg::EnableScreenShare(true)); 

        html! {
            <main class="px-3">
                <div class="row">
                    { render() }
                </div>
                <div class="row">
                    <div class="col">
                        <textarea 
                            id={ TEXTAREA_ID_CLIENT }
                            client_id={ client_id } 
                            ref={ self.client_area.clone() } 
                            class="document" cols="100" rows="30" 
                            { placeholder }
                            oninput={ oninput_client } 
                        />
                    </div>
                    <div class="col">
                        <textarea id={ TEXTAREA_ID } ref={ self.host_area.clone() } class="document" cols="100" rows="30" { placeholder } { oninput }/>
                    </div>
                </div>
                <video class="self-camera" autoplay=true id={VIDEO_ELEMENT_ID}></video>
                <DeviceSelector on_microphone_select={mic_callback} on_camera_select={cam_callback}/>
                <div>
                    <button onclick={ screen_share_cb }>{"Демонстрация экрана"}</button>
                </div>
            </main>
        }
    }

}

fn init(session_id: SessionId) -> HostManager {
    let connection_type = ConnectionType::StunAndTurn {
        stun_urls: env!("STUN_SERVER_URLS").to_string(),
        turn_urls: env!("TURN_SERVER_URLS").to_string(),
        username: env!("TURN_SERVER_USERNAME").to_string(),
        credential: env!("TURN_SERVER_CREDENTIAL").to_string(),
    };
    let signaling_server_url = concat!(env!("SIGNALING_SERVER_URL"), "/one-to-many");

    

    let mut host_manager = HostManager::new(session_id, connection_type, signaling_server_url);
    host_manager.init();
    host_manager
}
