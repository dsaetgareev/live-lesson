use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::str::FromStr;

use gloo_timers::callback::Timeout;
use wasm_bindgen::prelude::Closure;
use wasm_peers::one_to_many::MiniServer;
use wasm_peers::{get_random_session_id, ConnectionType, SessionId, UserId};
use yew::prelude::*;
use log::error;

use crate::components::multi::host::client_area::ClientArea;
use crate::components::multi::host::client_items::ClientItems;
use crate::components::multi::host::host_area::HostArea;
use crate::encoders::camera_encoder::CameraEncoder;
use crate::media_devices::device_selector::DeviceSelector;
use crate::components::multi::host::host_manager::HostManager;
use crate::models::client::{ClientProps, ClientItem};
use crate::models::commons::AreaKind;
use crate::models::host::HostPorps;
use crate::models::packet::VideoPacket;
use crate::utils::inputs::Message;
use crate::encoders::microphone_encoder::MicrophoneEncoder;
use crate::encoders::screen_encoder::ScreenEncoder;
use crate::utils::{self, dom::get_window};
use crate::wrappers::EncodedAudioChunkTypeWrapper;

const VIDEO_ELEMENT_ID: &str = "webcam";

pub enum Msg {
    Init,
    Tick,
    ChooseItem(String),
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
    host_manager: Option<Rc<RefCell<HostManager>>>,
    tick_callback: Closure<dyn FnMut()>,
    host_props: Rc<RefCell<HostPorps>>,
    client_props: Rc<RefCell<ClientProps>>,
    camera: CameraEncoder,
    microphone: MicrophoneEncoder,
    pub screen: ScreenEncoder,
    
}

impl Host {
    fn send_message_to_all(&mut self, message: &str) {
        self.host_manager
            .as_ref()
            .unwrap()
            .as_ref()
            .borrow()
            .mini_server
            .send_message_to_all(message)
            .expect("not send message")
    }

    fn send_message(&mut self, user_id: UserId, message: &str) {
        let _ = self.host_manager
            .as_ref()
            .expect("cannot get host manager")
            .borrow()
            .mini_server
            .send_message(user_id, message)
            .expect("not send message");
    }

    fn get_mini_server(&self) -> MiniServer {
        self.host_manager
            .as_ref()
            .expect("cannot get host manager")
            .borrow()
            .mini_server
            .clone()
    }

    fn get_players(&self) -> Rc<RefCell<HashMap<UserId, ClientItem>>> {
        self.host_manager
            .as_ref()
            .expect("cannot get host manager")
            .borrow()
            .players
            .clone()
    }

    fn send_message_cb(&self) -> Callback<(UserId, String)> {
        let ms = self.get_mini_server();
        let send_message = Callback::from(move |(user_id, message ): (UserId, String)| {
            ms.send_message(user_id, &message).expect("cannot send message");
        });
        send_message
    }

    fn send_message_all_cb(&self) -> Callback<String> {
        let ms = self.get_mini_server();
        let send_message = Callback::from(move |message: String| {
            ms.send_message_to_all(&message).expect("cannot send message");
        });
        send_message
    }
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
                self.host_manager = Some(Rc::new(RefCell::new(init(
                    self.session_id.clone(),
                    on_tick,
                    self.host_props.clone(),
                    self.client_props.clone(),
                ))));
                ctx.link().send_message(Msg::Tick);
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
                let client_item = self.get_players()
                    .borrow()
                    .get(&UserId::new(self.client_props.borrow().client_id.parse::<u64>().unwrap()))
                    .unwrap()
                    .clone();
                self.client_props.borrow_mut().set_area_kind(client_item.area_kind);
                self.client_props.borrow_mut().set_editor_content(client_item.editor_content);
                self.client_props.borrow_mut().set_text_area_content(client_item.text_area_content);
                self.client_props.borrow_mut().is_write = true;
                true
            },
            Self::Message::EnableVideo(should_enable) => {
                if !should_enable {
                    return true;
                }

                let ms = self.get_mini_server();
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

                
                let ms = self.get_mini_server();
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

                let ms = self.get_mini_server();
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

                let ms = self.get_mini_server();
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
                let message = serde_json::to_string(&Message::HostSwitchAudio).unwrap();
                let _ = self.send_message(UserId::new(client_id.parse::<u64>().unwrap()), &message);
                false
            }
            Msg::SwitchVideo(client_id) => {
                let message = serde_json::to_string(&Message::HostSwitchVideo).unwrap();
                let _ = self.send_message(UserId::new(client_id.parse::<u64>().unwrap()), &message);
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
                            players={ host_manager.as_ref().borrow().players.clone() } 
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

            match &self.host_manager {
                Some(_host_manager) => {
                    html! {
                        <HostArea 
                            host_props={ &self.host_props.clone() }
                            send_message_cb={ &self.send_message_cb() }
                            send_message_all_cb={ &self.send_message_all_cb() }
                        />
                    }
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

        let render_client = || {     
            
            match &self.host_manager {
                Some(_host_manager) => {
                    let area_kind = self.client_props.as_ref().borrow().client_area_kind;
                    let link = ctx.link().clone();
                    let on_tick = Callback::from(move |_: String| {
                        link.send_message(Msg::Tick);
                    });
                    html! {
                        <ClientArea
                            client_props={ &self.client_props.clone() }
                            send_message_cb={ &self.send_message_cb() }
                            players={ &self.get_players() }
                            on_tick={ on_tick }
                        />
                    }
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
                    <div>
                        <video class="client_canvas" autoplay=true id={VIDEO_ELEMENT_ID}></video>
                    </div>
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
