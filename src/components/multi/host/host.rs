use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::str::FromStr;

use gloo_timers::callback::Timeout;
use wasm_peers::one_to_many::MiniServer;
use wasm_peers::{get_random_session_id, SessionId, UserId};
use yew::prelude::*;
use log::error;
use yewdux::prelude::{Dispatch, use_store};

use crate::components::multi::host::client_area::ClientArea;
use crate::components::multi::host::client_items::{ClientItems, ClientBox};
use crate::components::multi::host::host_area::HostArea;
use crate::encoders::camera_encoder::CameraEncoder;
use crate::media_devices::device_selector::DeviceSelector;
use crate::components::multi::host::host_manager::HostManager;
use crate::models::client::{ClientProps, ClientItem};
use crate::models::host::HostPorps;
use crate::models::packet::{VideoPacket, AudioPacket};
use crate::stores::host_store::{HostStore, self};
use crate::utils::inputs::Message;
use crate::encoders::microphone_encoder::MicrophoneEncoder;
use crate::encoders::screen_encoder::ScreenEncoder;
use crate::utils;

const VIDEO_ELEMENT_ID: &str = "webcam";

pub enum Msg {
    Init,
    Tick,
    ChooseItem(String),
    SwitchSpeakers(String),
    SwitchVideo(String),
}

#[function_component(Devices)]
pub fn devices() -> Html {
    let (_state, dispatch) = use_store::<HostStore>();
    let mic_callback: Callback<String> = {
        let dispatch = dispatch.clone();
        Callback::from(move |audio| {
            dispatch.apply(host_store::Msg::AudioDeviceChanged(audio))
        })
    };
    let cam_callback = {
        let dispatch = dispatch.clone();
        Callback::from(move |video| {
            dispatch.apply(host_store::Msg::VideoDeviceChanged(video));
        })
    };
    html! {
        <>
            <DeviceSelector on_microphone_select={mic_callback} on_camera_select={cam_callback}/>
        </>
    }
}

#[function_component(ScreenShare)]
pub fn screen_share() -> Html {
    let (_state, dispatch) = use_store::<HostStore>();
    let screen_share_cb = {
        Callback::from(move |_| {
            dispatch.apply(host_store::Msg::EnableScreenShare(true));
        })
    };
    html! {
        <div>
            <button onclick={ screen_share_cb }>{"Демонстрация экрана"}</button>
        </div>
    }
}

pub struct Host {
    session_id: SessionId,
    host_manager: Option<Rc<RefCell<HostManager>>>,
    host_props: Rc<RefCell<HostPorps>>,
    client_props: Rc<RefCell<ClientProps>>,
    camera: CameraEncoder,
    microphone: MicrophoneEncoder,
    pub screen: ScreenEncoder,
    
}

impl Host {
    fn _send_message_to_all(&mut self, message: &Message) {
        self.host_manager
            .as_ref()
            .unwrap()
            .as_ref()
            .borrow()
            .mini_server
            .send_message_to_all(message)
            .expect("not send message")
    }

    fn send_message(&mut self, user_id: UserId, message: &Message) {
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

    fn send_message_cb(&self) -> Callback<(UserId, Message)> {
        let ms = self.get_mini_server();
        let send_message = Callback::from(move |(user_id, message ): (UserId, Message)| {
            ms.send_message(user_id, &message).expect("cannot send message");
        });
        send_message
    }

    fn send_message_all_cb(&self) -> Callback<Message> {
        let ms = self.get_mini_server();
        let send_message = Callback::from(move |message: Message| {
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

        ctx.link().send_message(Msg::Init);
        Self {
            session_id,
            host_manager: None,
            host_props: Rc::new(RefCell::new(HostPorps::new())),
            client_props: Rc::new(RefCell::new(ClientProps::new())),
            camera: CameraEncoder::new(),
            microphone: MicrophoneEncoder::new(),
            screen: ScreenEncoder::new(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Self::Message::Init => {
                // let link = ctx.link().clone();
                // let on_tick = move || {
                //     link.send_message(Msg::Tick)
                // };
                // // self.host_manager = Some(Rc::new(RefCell::new(init(
                // //     self.session_id.clone(),
                // //     on_tick,
                // //     self.host_props.clone(),
                // //     self.client_props.clone(),
                // // ))));
                // ctx.link().send_message(Msg::Tick);
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
            Msg::SwitchSpeakers(client_id) => {
                let _ = self.send_message(UserId::new(client_id.parse::<u64>().unwrap()), &Message::HostSwitchAudio);
                false
            }
            Msg::SwitchVideo(client_id) => {
                let _ = self.send_message(UserId::new(client_id.parse::<u64>().unwrap()), &Message::HostSwitchVideo);
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {

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

             log::error!("111");   
        html! {
            <div class="container">
                <div id="client-items" class="client-items">
                    <ClientItems />
                </div>
                <div class="host-content">
                    <div class="content-item">
                        <ClientArea />
                    </div>
                    <div class="content-item">
                        { render_host() }
                    </div>
                </div>
                <div class="host-video">
                    <Devices />
                    <ScreenShare />
                    <video class="client_canvas" autoplay=true id={VIDEO_ELEMENT_ID}></video>
                </div>
                       
                
                
            </div>
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
    let mut host_manager = HostManager::new(session_id);
    // host_manager.init(on_tick, host_content, client_props);
    host_manager
}
