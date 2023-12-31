use std::cell::RefCell;
use std::rc::Rc;
use std::str::FromStr;
use gloo_timers::callback::Timeout;
use wasm_peers::one_to_many::MiniClient;
use wasm_peers::{get_random_session_id, SessionId};
use yew::{html, Component, Context, Html, Callback};
use log::error;
use yew_icons::{Icon, IconId};

use crate::components::multi::client::client_area::ClientArea;
use crate::components::multi::client::host_area::HostArea;
use crate::encoders::camera_encoder::CameraEncoder;
use crate::encoders::microphone_encoder::MicrophoneEncoder;
use crate::models::audio::Audio;
use crate::models::client::ClientProps;
use crate::models::host::HostPorps;
use crate::models::packet::{VideoPacket, AudioPacket};
use crate::utils::device::create_audio_decoder;
use crate::utils::dom::{on_visible_el, get_vis_class};
use crate::utils::inputs::{ClientMessage, ManyMassage};
use crate::utils;
use crate::media_devices::device_selector::DeviceSelector;

use super::client_manager::ClientManager;

const VIDEO_ELEMENT_ID: &str = "webcam";

// #[derive(PartialEq, Properties)]
// pub struct ClientCmpPorps {
//     pub audio: Rc<RefCell<Audio>>,
// }

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
    audio: Rc<RefCell<Audio>>,
}

impl Client {
    pub fn get_mini_client(&self) -> MiniClient {
        self.client_manager
            .as_ref()
            .expect("cannot get client managet")
            .mini_client
            .clone()
    }
    pub fn send_message_to_host_cb(&self) -> Callback<ClientMessage> {
        let mc = self.get_mini_client();
        let send_message = Callback::from(move |message: ClientMessage | {
            mc.send_message_to_host(&message).expect("cannot send message");
        });
        send_message
    }
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
        let audio = Rc::new(RefCell::new(create_audio_decoder()));
        Self {
            session_id,
            client_manager: None,
            host_props: Rc::new(RefCell::new(HostPorps::new())),
            client_props: Rc::new(RefCell::new(ClientProps::new(String::default(), String::default()))),
            camera: CameraEncoder::new(),
            microphone: MicrophoneEncoder::new(),
            audio,
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
                    self.audio.clone(),
                ));
                true
            },
            Msg::Tick => {
                true
            }
            Msg::UpdateValue(content) => {
                self.client_props.borrow_mut().client_editor_content = content.clone();
                let message = ClientMessage::ClientText { message: content };
                let _ = self.client_manager.as_mut().unwrap().mini_client.send_message_to_host(&message);
                true
            },
            Msg::VideoDeviceChanged(video) => {
                if self.camera.select(video) {
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
                let is_video = !self.camera.get_enabled();
                on_visible_el(is_video, VIDEO_ELEMENT_ID, "video-logo");
                let message = ClientMessage::ClientSwitchVideo { message: is_video };
                let _ = self.client_manager.as_mut().unwrap().mini_client.send_message_to_host(&message);
                
                log::info!("{}", on_video);
                if self.camera.get_enabled() {
                    let timeout = Timeout::new(1000, move || {
                        link.send_message(Msg::EnableVideo(on_video));
                    });
                    timeout.forget();
                }
                true
            }
            Msg::EnableVideo(should_enable) => {
                if !should_enable {
                    return true;
                }

                let ms = self.client_manager.as_mut().unwrap().mini_client.clone();
                let nm = self.client_manager.as_ref().unwrap().network_manager.clone();
                let is_communication = self.host_props.borrow().is_communication.clone();
                let on_frame = move |packet: VideoPacket| {
                                       
                    let message = ClientMessage::ClientVideo { 
                        message: packet.clone()
                    };
                    let _ = ms.send_message_to_host(&message);
                    if *is_communication.borrow() {
                        let message = ManyMassage::Video { packet };
                        let _ = nm.send_message_to_all(&message);
                    }
                };
                self.camera.start(
                    on_frame,
                    VIDEO_ELEMENT_ID,
                );
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
                let nm = self.client_manager.as_mut().unwrap().network_manager.clone();
                let is_communication = self.host_props.borrow().is_communication.clone();
                let on_audio = move |chunk: web_sys::EncodedAudioChunk| {
                    
                    let audio_packet = AudioPacket::new(chunk);
                    let message = ClientMessage::ClientAudio { 
                        packet: audio_packet.clone()
                    };
                    let _ = ms.send_message_to_host(&message);
                    if *is_communication.borrow() {
                        let message = ManyMassage::Audio { 
                            packet: audio_packet
                        };
                        let _ = nm.send_message_to_all(&message);
                    }                                
                };
                self.microphone.start(
                    on_audio
                );
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let mic_callback = ctx.link().callback(Msg::AudioDeviceChanged);
        let cam_callback = ctx.link().callback(Msg::VideoDeviceChanged);
        let on_video_btn = ctx.link().callback(|_| Msg::SwitchVedeo);

        let render_host_area = || {
            html! {
                <HostArea 
                    host_props={ self.host_props.clone() } 
                    area_kind={ self.host_props.clone().borrow().host_area_kind }
                    editor_content={ self.host_props.as_ref().borrow().host_editor_content.clone() }
                    text_area_content={ self.host_props.clone().borrow().host_area_content.content.clone() }
                />
            }
        };

        let render_client_area = || {

            match &self.client_manager {
                Some(_client_manager) => {
                    html! {
                        <ClientArea 
                            client_props={ &self.client_props.clone() }
                            send_message_to_host_cb={ &self.send_message_to_host_cb() }
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

        let is_visible = get_vis_class(*(self.host_props.borrow().is_communication.borrow())); 

        html! {
            <div id="container" class="container">
                <div class="client-container">
                    <div class="content-item">
                        <div>
                            <button onclick={ on_video_btn }>
                                { 
                                    if self.camera.get_enabled() {
                                        html! { <Icon icon_id={IconId::BootstrapCameraVideoOffFill}/> }
                                    } else {
                                        html! { <Icon icon_id={IconId::BootstrapCameraVideoFill}/> }
                                    }
                                }
                                
                            </button>
                        </div>
                        <video class="client_canvas vis" autoplay=true id={VIDEO_ELEMENT_ID} poster="placeholder.png"></video>
                        <div id="video-logo" class="unvis">
                            <Icon icon_id={IconId::FontAwesomeSolidHorseHead}/>
                        </div>
                        <div id="video-box" class={ is_visible }>
                        </div>
                    </div>
                    <div class=".content-item">
                        { render_client_area() }
                    </div>
                    <div class=".content-item">
                        { render_host_area() }
                        
                    </div>
                    <div class=".content-item">                                             
                        <video id="render" autoplay=true class="client_canvas"></video>
                    </div>
                    
                </div>
                <DeviceSelector on_microphone_select={mic_callback} on_camera_select={cam_callback}/>

                <div id="shcreen_container" class="consumer unvis">
                    <video id="screen_share" autoplay=true class="screen_canvas"></video>
                </div>
                
            </div>
            
        }
    }

}

fn init(
    session_id: SessionId,
    on_tick: impl Fn() + 'static,
    host_content: Rc<RefCell<HostPorps>>,
    client_props: Rc<RefCell<ClientProps>>,
    audio: Rc<RefCell<Audio>>,
) -> ClientManager {

    let mut client_manager = ClientManager::new(session_id, audio);
    client_manager.init(on_tick, host_content, client_props);
    client_manager.many_init();
    client_manager
}