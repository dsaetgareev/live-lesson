use std::cell::RefCell;
use std::rc::Rc;
use std::str::FromStr;

use gloo_timers::callback::Timeout;
use monaco::api::TextModel;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::Closure;
use wasm_peers::{get_random_session_id, ConnectionType, SessionId, UserId};
use web_sys::{HtmlElement, MouseEvent};
use yew::prelude::*;
use log::error;

use crate::components::editor::editor::EditorWrapper;
use crate::components::multi::client_items::ClientItems;
use crate::encoders::camera_encoder::CameraEncoder;
use crate::media_devices::device_selector::DeviceSelector;
use crate::components::multi::host_manager::HostManager;
use crate::models::client::ClientProps;
use crate::models::host::HostPorps;
use crate::models::packet::VideoPacket;
use crate::utils::dom::create_video_id;
use crate::utils::inputs::Message;
use crate::encoders::microphone_encoder::MicrophoneEncoder;
use crate::encoders::screen_encoder::ScreenEncoder;
use crate::utils::{self, dom::get_window};
use crate::wrappers::EncodedAudioChunkTypeWrapper;

const TEXTAREA_ID: &str = "document-textarea";
const TEXTAREA_ID_CLIENT: &str = "client-textarea";
const VIDEO_ELEMENT_ID: &str = "webcam";

pub enum Msg {
    Init,
    UpdateValue(String),
    Tick,
    ChooseItem(String),
    SendClient(String),
    EnableVideo(bool),
    EnableMicrophone(bool),
    EnableScreenShare(bool),
    AudioDeviceChanged(String),
    VideoDeviceChanged(String),
    ResumeVideo,
    SwitchSpeakers(String),
    SwitchVideo(String),
}

pub struct Host {
    session_id: SessionId,
    host_manager: Option<HostManager>,
    tick_callback: Closure<dyn FnMut()>,
    host_props: Rc<RefCell<HostPorps>>,
    client_props: Rc<RefCell<ClientProps>>,
    camera: CameraEncoder,
    microphone: MicrophoneEncoder,
    pub screen: ScreenEncoder,
    
}

impl Component for Host {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let query_params = utils::dom::get_query_params_multi();
        let (session_id, _is_host) =
            match (query_params.get("session_id"), query_params.get("is_host")) {
                (Some(session_string), Some(is_host)) => {
                    (SessionId::new(uuid::Uuid::from_str(&session_string).unwrap().as_u128()), is_host == "true")
                }
                _ => {
                    let location = utils::dom::global_window().location();
                    let generated_session_id = get_random_session_id();
                    query_params.append("session_id", &generated_session_id.to_string());
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
        ctx.link().send_message(Msg::Init);
        Self {
            session_id,
            host_manager: None,
            tick_callback,
            host_props: Rc::new(RefCell::new(HostPorps::new())),
            client_props: Rc::new(RefCell::new(ClientProps::new(String::default(), String::default()))),
            camera: CameraEncoder::new(),
            microphone: MicrophoneEncoder::new(),
            screen: ScreenEncoder::new(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Self::Message::Init => {
                let link = ctx.link().clone();
                let on_tick = move || {
                    link.send_message(Msg::Tick)
                };
                self.host_manager = Some(init(
                    self.session_id.clone(),
                    on_tick,
                    self.host_props.clone(),
                    self.client_props.clone(),
                ));
                ctx.link().send_message(Msg::Tick);
                false
            },
            Self::Message::UpdateValue(content) => {
                self.host_props.borrow_mut().host_content = content;

                let message = Message::HostToHost {
                             message: self.host_props.borrow().host_content.clone()
                };
                let message = serde_json::to_string(&message).unwrap();
                let _ = self.host_manager.as_ref().unwrap().mini_server.send_message_to_all(&message).expect("not send message");
                false            
            },
            Self::Message::Tick => {
                // if let Err(error) = get_window().unwrap().request_animation_frame(
                //     self.tick_callback.as_ref().unchecked_ref(),
                // ) {
                //     error!("Failed requesting next animation frame: {error:?}");
                // }
                true
            },
            Self::Message::ChooseItem(client_id) => {
                self.client_props.borrow_mut().client_id = client_id;
                let value = self.host_manager
                    .as_ref()
                    .unwrap()
                    .players
                    .borrow()
                    .get(&UserId::new(self.client_props.borrow().client_id.parse::<u64>().unwrap()))
                    .unwrap()
                    .clone();
                log::error!("value {}", value);
                self.client_props.borrow_mut().client_content = value;
                self.client_props.borrow_mut().is_write = true;
                true
            },
            Self::Message::SendClient(content) => {
                self.client_props.borrow_mut().client_content = content.clone();
                if self.client_props.borrow().client_id.is_empty() {
                    false
                } else {
                    let user_id: UserId = UserId::new(self.client_props.borrow().client_id.parse::<u64>().unwrap());
                    self.host_manager.as_ref().unwrap().players.borrow_mut().insert(user_id, content.clone()); 
                    let message = Message::HostToClient {
                        message: content.clone()
                    };
                    let message = serde_json::to_string(&message).unwrap();
                    let _ = self.host_manager.as_ref().unwrap().mini_server.send_message(user_id, &message);
                    self.client_props.borrow_mut().is_write = false;
                    true
                }
            },
            Self::Message::EnableVideo(should_enable) => {
                if !should_enable {
                    return true;
                }

                let ms = self.host_manager.as_ref().unwrap().mini_server.clone();
                let on_frame = move |packet: VideoPacket| {
                    let message = Message::HostVideo { 
                        message: packet
                    };
                    match serde_json::to_string(&message) {
                        Ok(message) => {
                            let _ = ms.send_message_to_all(&message);
                        },
                        Err(_) => todo!(),
                    };
                    
                };
                self.camera.start(
                    on_frame,
                    VIDEO_ELEMENT_ID,
                );
                false
            },
            Self::Message::EnableScreenShare(should_enable) => {
               
                if !should_enable {
                    return true;
                }
                self.camera.set_enabled(false);

                
                let ms = self.host_manager.as_ref().unwrap().mini_server.clone();
                match serde_json::to_string(&Message::HostIsScreenShare { message: true }) {
                    Ok(message) => {
                        let _ = ms.send_message_to_all(&message);
                    },
                    Err(_) => todo!(),
                }
                let on_frame = move |packet: VideoPacket| {
                    
                    let message = Message::HostScreenShare { 
                        message: packet,
                    };
                    match serde_json::to_string(&message) {
                        Ok(message) => {
                            let _ = ms.send_message_to_all(&message);
                        },
                        Err(_) => todo!(),
                    };                    
                };

                let ms = self.host_manager.as_ref().unwrap().mini_server.clone();
                let link = ctx.link().clone();
                let on_stop_share = move || {
                    link.send_message(Msg::ResumeVideo);
                    link.send_message(Msg::EnableVideo(true));
                    let message = Message::HostIsScreenShare { message: false };
                    match serde_json::to_string(&message) {
                        Ok(message) => {
                            let _ = ms.send_message_to_all(&message);
                        },
                        Err(_) => todo!(),
                    };   
                };
                self.screen.start(
                    on_frame,
                    on_stop_share,
                );
                false
            },
            Self::Message::ResumeVideo => {
                self.camera.set_enabled(true);
                false
            },
            Self::Message::EnableMicrophone(should_enable) => {
                if !should_enable {
                    return true;
                }

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
                            let _ = ms.send_message_to_all(&message);
                        },
                        Err(_) => todo!(),
                    };                    
                };              
                self.microphone.start(
                    on_audio
                );
                false
            }
            Msg::AudioDeviceChanged(audio) => {
                if self.microphone.select(audio) {
                    let link = ctx.link().clone();
                    let timeout = Timeout::new(1000, move || {
                        link.send_message(Msg::EnableMicrophone(true));
                    });
                    timeout.forget();
                }
                false
            }
            Msg::VideoDeviceChanged(video) => {
                if self.camera.select(video) {
                    let link = ctx.link().clone();
                    let timeout = Timeout::new(1000, move || {
                        link.send_message(Msg::EnableVideo(true));
                    });
                    timeout.forget();
                }
                false
            }
            Msg::SwitchSpeakers(client_id) => {
                let message = serde_json::to_string(&Message::HostSwicthAudio).unwrap();
                let _ = self.host_manager.as_ref().unwrap().mini_server.send_message(UserId::new(client_id.parse::<u64>().unwrap()), &message);
                false
            }
            Msg::SwitchVideo(client_id) => {
                let message = serde_json::to_string(&Message::HostSwicthVideo).unwrap();
                let _ = self.host_manager.as_ref().unwrap().mini_server.send_message(UserId::new(client_id.parse::<u64>().unwrap()), &message);
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {

        let render_items = || {
            match self.host_manager.as_ref() {
                Some(host_manager) => {
                    let on_switch_speakers = ctx.link().callback(move |speakers_id: String| Self::Message::SwitchSpeakers(speakers_id));
                    let on_switch_video = ctx.link().callback(move |video_switch_id: String|  Msg::SwitchVideo(video_switch_id));
                    let on_choose_item = ctx.link().callback(|client_id: String| Msg::ChooseItem(client_id));
                    html!(
                        <ClientItems 
                            players={ host_manager.players.clone() } 
                            on_switch_speakers={ on_switch_speakers }
                            on_switch_video={ on_switch_video }
                            on_choose_item={ on_choose_item }
                        />
                    )            
                },
                None => {
                    html!(
                        <div>
                            {"none host manager"}
                        </div>
                    )                
                },
            }
        };

        let render_host = || {
            let text_model = TextModel::create(&self.host_props.borrow().host_content, Some("rust"), None).unwrap();
            let on_host_editor_cb = ctx.link().callback(|content: String| Msg::UpdateValue(content));

            html! {
                <div class="col document">
                    <EditorWrapper on_cb={ on_host_editor_cb } text_model={ text_model.clone() } is_write={ true }/>
                </div>
            }
        };

        let render_client = || {
            let text_model_client = TextModel::create(&self.client_props.borrow().client_content, Some("rust"), None).unwrap();
            let on_client_editor_cb = ctx.link().callback(|content: String| Self::Message::SendClient(content));
        
            html! {
                <div class="col document">
                    <EditorWrapper on_cb={ on_client_editor_cb } text_model={ text_model_client } is_write={ self.client_props.borrow().is_write }/>
                </div>
            }
        };
        
        let mic_callback = ctx.link().callback(Msg::AudioDeviceChanged);
        let cam_callback = ctx.link().callback(Msg::VideoDeviceChanged);

        let screen_share_cb = ctx.link().callback(|_| Msg::EnableScreenShare(true)); 
        
        html! {
            <main class="">
                <div class="row">
                    { render_items() }
                </div>
                <div class="row">
                    { render_client() }
                    { render_host() }
                </div>
                <div class="consumer">
                    <h3>{"Consumer!"}</h3>
                    <video class="self-camera" autoplay=true id={VIDEO_ELEMENT_ID}></video>
                </div>
                
                <DeviceSelector on_microphone_select={mic_callback} on_camera_select={cam_callback}/>
                <div>
                    <button onclick={ screen_share_cb }>{"Демонстрация экрана"}</button>
                </div>
                
            </main>
        }
    }

    fn destroy(&mut self, _ctx: &Context<Self>) {
        log::debug!("destroying");
        self.camera.stop();
        self.microphone.stop();
        self.screen.stop();
    }

}

fn init(
    session_id: SessionId,
    on_tick: impl Fn() + 'static,
    host_content: Rc<RefCell<HostPorps>>,
    client_props: Rc<RefCell<ClientProps>>,
) -> HostManager {
    let connection_type = ConnectionType::StunAndTurn {
        stun_urls: env!("STUN_SERVER_URLS").to_string(),
        turn_urls: env!("TURN_SERVER_URLS").to_string(),
        username: env!("TURN_SERVER_USERNAME").to_string(),
        credential: env!("TURN_SERVER_CREDENTIAL").to_string(),
    };
    let signaling_server_url = concat!(env!("SIGNALING_SERVER_URL"), "/one-to-many");

    let mut host_manager = HostManager::new(session_id, connection_type, signaling_server_url);
    host_manager.init(on_tick, host_content, client_props);
    host_manager
}
