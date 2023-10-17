use std::{cell::RefCell, rc::Rc, collections::HashMap};

use gloo_timers::callback::Timeout;
use wasm_bindgen::JsCast;
use wasm_peers::{SessionId, UserId, one_to_many::MiniServer};
use web_sys::{MouseEvent, HtmlElement, InputEvent, HtmlTextAreaElement};
use yew::Callback;
use yewdux::{store::{Reducer, Store}, prelude::Dispatch};

use crate::{components::multi::{host::host_manager::HostManager, draw}, encoders::{microphone_encoder::MicrophoneEncoder, camera_encoder::CameraEncoder, screen_encoder::ScreenEncoder}, models::{host::HostPorps, client::{ClientProps, ClientItem}, packet::{AudioPacket, VideoPacket}, commons::{AreaKind, InitUser}, video::Video, audio::Audio}, stores::host_store, utils::{inputs::Message, dom::{create_video_id, on_visible_el}}, constants::VIDEO_ELEMENT_ID};

#[derive(Clone, PartialEq, Store)]
pub struct HostStore {
    pub session_id: Option<SessionId>,
    pub host_props: Option<HostPorps>,
    pub client_props: Option<ClientProps>,
    pub host_manager: Option<Rc<RefCell<HostManager>>>,
    pub camera: Option<CameraEncoder>,
    pub microphone: Option<MicrophoneEncoder>,
    pub screen: Option<ScreenEncoder>,
    pub players: HashMap<UserId, ClientItem>,
}

impl Default for HostStore {
    fn default() -> Self {
        Self { 
            session_id: None,
            host_props:None,
            client_props: None,
            host_manager: None,
            camera: None,
            microphone: None,
            screen: None,
            players: HashMap::new(),
        }
    }
}

impl HostStore {
    pub fn init(&mut self, session_id: SessionId) {
        let host_props = HostPorps::new();
        let client_props = ClientProps::new();
        let mut host_manager = HostManager::new(session_id);
        let dispatch = Dispatch::<HostStore>::new();
        let on_action = move |msg: host_store::Msg| {
            dispatch.apply(msg);
        };
        host_manager.init(on_action);
        self.session_id = Some(session_id);
        self.host_props = Some(host_props);
        self.client_props = Some(client_props);
        self.host_manager = Some(Rc::new(RefCell::new(host_manager)));
        self.camera = Some(CameraEncoder::new());
        self.microphone = Some(MicrophoneEncoder::new());
        self.screen = Some(ScreenEncoder::new());
    }

    pub fn get_mini_server(&self) -> MiniServer {
        self.host_manager
            .as_ref()
            .expect("cannot get host manager")
            .borrow()
            .mini_server
            .clone()
    }

    pub fn get_players(&self) -> Rc<RefCell<HashMap<UserId, ClientItem>>> {
        Rc::clone(&self.host_manager
            .as_ref()
            .expect("cannot get host manager")
            .borrow()
            .players)
    }

    pub fn get_decoders(&self) -> Rc<RefCell<HashMap<UserId, Rc<RefCell<Video>>>>> {
        self.host_manager
            .as_ref()
            .expect("cannot get host manager")
            .borrow()
            .video_decoders
            .clone()
    }

    pub fn get_audio_decoders(&self) -> Rc<RefCell<HashMap<UserId, Rc<RefCell<Audio>>>>> {
        self.host_manager
            .as_ref()
            .expect("cannot get host manager")
            .borrow()
            .audio_decoders
            .clone()
    }

    pub fn get_host_props(&self) -> &HostPorps {
        self.host_props.as_ref().unwrap()
    }

    pub fn get_mut_host_props(&mut self) -> &mut HostPorps {
        self.host_props.as_mut().unwrap()
    }

    pub fn get_client_props(&self) -> &ClientProps {
        self.client_props.as_ref().unwrap()
    }

    pub fn get_mut_client_props(&mut self) -> &mut ClientProps {
        self.client_props .as_mut().unwrap()
    }

    pub fn send_message_to_all(&self, mesage: Message) {
        let _ = self.get_mini_server().send_message_to_all(&mesage);
    }
}

#[derive(PartialEq)]
pub enum Msg {
    Init(SessionId),
    AudioDeviceChanged(String),
    EnableMicrophone(bool),
    VideoDeviceChanged(String),
    EnableVideo(bool),
    EnableScreenShare(bool),
    ResumeVideo,
    HostClientToClient(String),
    //ClientItems actions
    ChooseItem(MouseEvent),
    SwitchSpeakers(String),
    SwitchVideo(String),
    // Host manager actions
    AddClient(UserId),
    DisconnectClient(UserId),
    InitClient(UserId, InitUser),
    ClientSwitchVideo(UserId, bool),
    ClientToClient(UserId, String, AreaKind),
    ClientSwitchArea(UserId, AreaKind),
    HostUpdateValue(String),
    HostTextAreaInput(InputEvent),
    SwitchHostArea(AreaKind),
    OpenPaint,
    OnCummunication,
}

impl Reducer<HostStore> for Msg {
    fn apply(self, mut store: Rc<HostStore>) -> Rc<HostStore> {
        let state = Rc::make_mut(&mut store);
        let dispatch = Dispatch::<HostStore>::new();

        match self {
            Msg::Init(session_id) => {
                state.init(session_id);
            }
            Msg::AddClient(user_id) => {
                let editor_content = state.get_host_props().host_editor_content.clone();
                let text_area_content = state.get_host_props().host_area_content.content.clone();
                let area_kind = state.get_host_props().host_area_kind;
                let is_communication = state.get_host_props().is_communication;
                let init_user = InitUser {
                    editor_content,
                    text_area_content,
                    area_kind: area_kind.clone(),
                    is_communication
                };
                let message = Message::Init { 
                    message: init_user,      
                };
                state.get_mini_server()
                    .send_message(user_id, &message)
                    .expect("failed to send current input to new connection");

                state.players
                    .insert(user_id, ClientItem::new(AreaKind::TextArea));
            }
            Msg::InitClient(user_id, init_user) => {
                let client_item = state.players.get_mut(&user_id).unwrap();
                client_item.set_area_kind(init_user.area_kind);
                client_item.set_editor_content(init_user.editor_content);
                client_item.set_text_area_content(init_user.text_area_content);
            }
            Msg::DisconnectClient(user_id) => {
                log::error!("disconect {}", user_id);
                match state.players.get(&user_id) {
                    Some(_client_item) => {
                        state.players
                            .remove(&user_id)
                            .expect("cannot remove clietn");
                    },
                    None => {
                        log::error!("not found client id: {}", user_id);
                    },
                }
            }
            Msg::ClientSwitchVideo(user_id, message) => {
                let video_id = create_video_id(user_id.to_string());
                let client_logo_id = create_video_id(format!("{}_{}", "client-video-logo", user_id.to_string()));
                on_visible_el(message, &video_id, &client_logo_id);
            }
            Msg::ClientToClient(
                user_id,
                message,
                area_kind
            ) => {
                match state.players.get_mut(&user_id) {
                    Some(client_item) => {
                        client_item.set_area_kind(area_kind);
                        match area_kind {
                            AreaKind::Editor => {
                                client_item.set_editor_content(message.clone());
                                if state.get_client_props().client_id == user_id.to_string() {
                                    state.get_mut_client_props().set_editor_content(message);
                                    state.get_mut_client_props().is_write = true;
                                }
                            },
                            AreaKind::TextArea => {
                                client_item.set_text_area_content(message.clone());
                                if state.get_client_props().client_id == user_id.to_string() {
                                    state.get_mut_client_props().set_text_area_content(message);
                                }
                            },
                        }
                        
                    },
                    None => {
                        log::error!("cannot find client item, id: {}", user_id.to_string());
                    },
                }
            }
            Msg::ClientSwitchArea(user_id, area_kind) => {
                log::error!("switck {:?}", area_kind);
                if state.get_client_props().client_id == user_id.to_string() {
                    state.get_mut_client_props().set_area_kind(area_kind);
                }

                match state.players.get_mut(&user_id) {
                    Some(client_item) => {
                        client_item.set_area_kind(area_kind)
                    },
                    None => todo!(),
                }
                
            }
            Msg::HostUpdateValue(content) => {
                let host_area_kind = state.get_host_props().host_area_kind;
                match host_area_kind {
                    AreaKind::Editor => {
                        state.get_mut_host_props().host_editor_content = content.clone();
                    },
                    AreaKind::TextArea => {
                        state.get_mut_host_props().host_area_content.set_content(content.clone());
                    },
                }                

                let message = Message::HostToHost {
                             message: content,
                             area_kind: state.get_host_props().host_area_kind
                };
                let ms = state.get_mini_server();
                let _ = ms.send_message_to_all(&message);
            }
            Msg::HostTextAreaInput(event) => {
                let content = event
                    .target()
                    .unwrap()
                    .unchecked_into::<HtmlTextAreaElement>()
                    .value();
                dispatch.apply(host_store::Msg::HostUpdateValue(content));
            }
            Msg::SwitchHostArea(area_kind) => {
                state.get_mut_host_props().set_host_area_kind(area_kind);

                let message = Message::HostSwitchArea { message: area_kind };
                let ms = state.get_mini_server();
                let _ = ms.send_message_to_all(&message);
            }
            Msg::OpenPaint => {
                let area_kind = state.get_host_props().host_area_kind;
                let send_message_all_cb = {
                    let ms = state.get_mini_server();
                    Callback::from(move |message: Message| {
                        let _ = ms.send_message_to_all(&message);
                    })
                };
                match area_kind {
                    AreaKind::Editor => {           
                        let _ = draw::paint::start(&state.get_host_props().host_editor_content, send_message_all_cb.clone());
                    },
                    AreaKind::TextArea => {
                        let _ = draw::paint::start(&state.get_host_props().host_area_content.content, send_message_all_cb.clone());
                    },
                }
                let message = Message::OpenPaint;
                let ms = state.get_mini_server();
                let _ = ms.send_message_to_all(&message);
            }
            Msg::OnCummunication => {
                let is_communication = state.get_mut_host_props().switch_communication();
                let message = Message::OnCummunication { message: is_communication };
                state.send_message_to_all(message);
            }




            Msg::AudioDeviceChanged(audio) => {
                if state.microphone.as_mut().unwrap().select(audio) {
                    let timeout = Timeout::new(1000, move || {
                        dispatch.apply(host_store::Msg::EnableMicrophone(true));
                    });
                    timeout.forget();
                }
            },
            Msg::EnableMicrophone(should_enable) => {
                if should_enable {
                    let ms = state.host_manager.as_ref().unwrap().borrow().mini_server.clone();
                    let on_audio = move |chunk: web_sys::EncodedAudioChunk| {
                        
                        let audio_packet = AudioPacket::new(chunk);
                        let message = Message::HostAudio { 
                            packet: audio_packet
                        };
                        let _ = ms.send_message_to_all(&message);                
                    };              
                    state.microphone.as_mut().unwrap().start(
                        on_audio
                    );
                }                
            }
            Msg::VideoDeviceChanged(video) => {
                if state.camera.as_mut().unwrap().select(video) {
                    let timeout = Timeout::new(1000, move || {
                        dispatch.apply(host_store::Msg::EnableVideo(true));
                    });
                    timeout.forget();
                }
            },
            Msg::EnableVideo(should_enable) => {
                if should_enable {
                    let ms = state.host_manager.as_ref().unwrap().borrow().mini_server.clone();
                    let on_frame = move |packet: VideoPacket| {
                        let message = Message::HostVideo { 
                            message: packet
                        };
                        let _ = ms.send_message_to_all(&message);
                        
                    };
                    state.camera.as_mut().unwrap().start(
                        on_frame,
                        VIDEO_ELEMENT_ID,
                    );
                }

                
            },
            Msg::EnableScreenShare(should_enable) => {

                if should_enable {
                    // state.camera.as_mut().unwrap().set_enabled(false); todo

                
                    let ms = state.host_manager.as_ref().unwrap().borrow().mini_server.clone();
                    let _ = ms.send_message_to_all(&Message::HostIsScreenShare { message: true });
                    let on_frame = move |packet: VideoPacket| {
                        
                        let message = Message::HostScreenShare { 
                            message: packet,
                        };
                        let _ = ms.send_message_to_all(&message);                 
                    };

                    let ms = state.host_manager.as_ref().unwrap().borrow().mini_server.clone();
                    let on_stop_share = move || {
                        dispatch.apply(host_store::Msg::ResumeVideo);
                        let message = Message::HostIsScreenShare { message: false };
                        let _ = ms.send_message_to_all(&message);
                    };
                    state.screen.as_mut().unwrap().start(
                        on_frame,
                        on_stop_share,
                    );
                }
            },
            Msg::ResumeVideo => {
                log::error!("resu;me");
                state.camera.as_mut().unwrap().set_enabled(true);
            },
            Msg::HostClientToClient(content) => {
                let client_id = state.get_client_props().client_id.clone();
                if !client_id.is_empty() {
                    let user_id: UserId = UserId::new(client_id.parse::<u64>().unwrap());
                    let area_kind = state.get_client_props().client_area_kind;
                    // let client_item = self.players.borrow_mut().get(&user_id);
                    match state.players.get_mut(&user_id) {
                        Some(client_item) => {
                            match area_kind {
                                AreaKind::Editor => {
                                    client_item.set_editor_content(content.clone());
                                    state.get_mut_client_props().set_editor_content(content.clone());
                                },
                                AreaKind::TextArea => {
                                    client_item.set_text_area_content(content.clone());
                                    state.get_mut_client_props().set_text_area_content(content.clone());
                                },
                            }
                        },
                        None => {
                            log::error!("cannot find client item, id: {}", user_id.to_string());
                        },
                    }
                                        
                    let message = Message::HostToClient {
                        message: content,
                        area_kind,
                    };
                    let ms = state.get_mini_server();
                    log::error!("user_id {}", user_id);
                    let _ = ms.send_message(user_id, &message);
                    state.get_mut_client_props().is_write = false;
                }
            }
            Msg::ChooseItem(event) => {
                let target: HtmlElement = event
                    .target()
                    .unwrap()
                    .dyn_into()
                    .unwrap();
                let client_id = target.get_attribute("client_id").unwrap();
                state.get_mut_client_props().set_client_id(client_id.clone());
                let client_item = state
                    .players
                    .get(&UserId::new(client_id.parse::<u64>().unwrap()))
                    .unwrap()
                    .clone();
                state.get_mut_client_props().set_area_kind(client_item.area_kind);
                state.get_mut_client_props().set_editor_content(client_item.editor_content);
                state.get_mut_client_props().set_text_area_content(client_item.text_area_content);
                state.get_mut_client_props().is_write = true;
            }
            Msg::SwitchSpeakers(_speakers_id) => {
                
            },
            Msg::SwitchVideo(_speakers_id) => {
                
            },
            
        };

        store
    }

}