use std::{cell::RefCell, rc::Rc, collections::HashMap};

use gloo_timers::callback::Timeout;
use wasm_bindgen::JsCast;
use wasm_peers::{SessionId, UserId, one_to_many::MiniServer};
use web_sys::{MouseEvent, HtmlElement};
use yewdux::{store::{Reducer, Store}, prelude::Dispatch};

use crate::{components::multi::host::host_manager::HostManager, encoders::{microphone_encoder::MicrophoneEncoder, camera_encoder::CameraEncoder, screen_encoder::ScreenEncoder}, models::{host::HostPorps, client::{ClientProps, ClientItem}, packet::{AudioPacket, VideoPacket}, commons::AreaKind, video::Video, audio::Audio}, stores::host_store, utils::{inputs::Message, dom::create_video_id, device::{create_video_decoder_video, VideoElementKind, create_audio_decoder}}};

const VIDEO_ELEMENT_ID: &str = "webcam";

#[derive(Clone, PartialEq, Store)]
pub struct HostStore {
    pub session_id: Option<SessionId>,
    pub host_props: Option<Rc<RefCell<HostPorps>>>,
    pub client_props: Option<Rc<RefCell<ClientProps>>>,
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
        let host_props = Rc::new(RefCell::new(HostPorps::new()));
        let client_props = Rc::new(RefCell::new(ClientProps::new()));
        let mut host_manager = HostManager::new(session_id);
        let dispatch = Dispatch::<HostStore>::new();
        let on_tick = move |msg: host_store::Msg| {
            log::debug!("on tick");
            dispatch.apply(msg);
        };
        host_manager.init(on_tick, host_props.clone(), client_props.clone());
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
            .decoders
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

    pub fn get_host_props(&self) -> Rc<RefCell<HostPorps>> {
        self.host_props
            .as_ref()
            .expect("no host props")
            .clone()
    }
}

pub enum Msg {
    Init(SessionId),
    Tick,
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
}

impl Reducer<HostStore> for Msg {
    fn apply(self, mut counter: Rc<HostStore>) -> Rc<HostStore> {
        let state = Rc::make_mut(&mut counter);
        let dispatch = Dispatch::<HostStore>::new();

        match self {
            Msg::Init(session_id) => {
                state.init(session_id);
            }
            Msg::AddClient(user_id) => {
                let editor_content = state.get_host_props().borrow().host_editor_content.clone();
                let text_area_content = state.get_host_props().borrow().host_area_content.content.clone();
                let area_kind = state.get_host_props().borrow().host_area_kind;
                let is_communication = *(state.get_host_props().borrow().is_communication.borrow());
                let message = Message::Init { 
                    editor_content,
                    text_area_content,
                    area_kind: area_kind.clone(),
                    is_communication
                };
                state.get_mini_server()
                    .send_message(user_id, &message)
                    .expect("failed to send current input to new connection");
                state.get_players()
                    .as_ref()
                    .borrow_mut()
                    .insert(user_id, ClientItem::new(area_kind));

                state.players
                    .insert(user_id, ClientItem::new(area_kind));

                let video_id = create_video_id(user_id.into_inner().to_string());
                state.get_decoders()
                    .as_ref()
                    .borrow_mut()
                    .insert(user_id, Rc::new(RefCell::new(create_video_decoder_video(video_id, VideoElementKind::HostBox))));
                state.get_audio_decoders()
                    .as_ref()
                    .borrow_mut()
                    .insert(user_id, Rc::new(RefCell::new(create_audio_decoder())));
                log::error!("clent {} added", user_id.to_string());
            }
            Msg::Tick => {
                state.host_manager
                    .as_ref()
                    .unwrap()
                    .borrow()
                    .players
                    .borrow()
                    .clone()
                    .into_keys()
                    .for_each(|item| {
                        log::error!("item {}", item.to_string());
                    });
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
                let client_id = &state.client_props.as_ref().unwrap().borrow().client_id.clone();
                if !client_id.is_empty() {
                    let user_id: UserId = UserId::new(client_id.parse::<u64>().unwrap());
                    let area_kind = state.client_props.as_ref().unwrap().borrow().client_area_kind;
                    // let client_item = self.players.borrow_mut().get(&user_id);
                    match state.host_manager.as_mut().unwrap().borrow().players.borrow_mut().get_mut(&user_id) {
                        Some(client_item) => {
                            match area_kind {
                                AreaKind::Editor => {
                                    client_item.set_editor_content(content.clone());
                                    state.client_props.clone().unwrap().borrow_mut().set_editor_content(content.clone());
                                },
                                AreaKind::TextArea => {
                                    client_item.set_text_area_content(content.clone());
                                    state.client_props.clone().unwrap().borrow_mut().set_text_area_content(content.clone());
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
                    let ms = &state.host_manager.as_ref().unwrap().borrow().mini_server;
                    log::error!("user_id {}", user_id);
                    let _ = ms.send_message(user_id, &message);
                    state.client_props.clone().unwrap().borrow_mut().is_write = false;
                }
            }
            Msg::ChooseItem(event) => {
                let target: HtmlElement = event
                    .target()
                    .unwrap()
                    .dyn_into()
                    .unwrap();
                let client_id = target.get_attribute("client_id").unwrap();
                state.client_props.as_ref().unwrap().borrow_mut().client_id = client_id.clone();
                let client_item = state.host_manager
                    .as_ref()
                    .unwrap()
                    .borrow()
                    .players
                    .borrow()
                    .get(&UserId::new(client_id.parse::<u64>().unwrap()))
                    .unwrap()
                    .clone();
                state.client_props.as_mut().unwrap().borrow_mut().set_area_kind(client_item.area_kind);
                state.client_props.as_mut().unwrap().borrow_mut().set_editor_content(client_item.editor_content);
                state.client_props.as_mut().unwrap().borrow_mut().set_text_area_content(client_item.text_area_content);
                state.client_props.as_mut().unwrap().borrow_mut().is_write = true;
            }
            Msg::SwitchSpeakers(speakers_id) => {
                
            },
            Msg::SwitchVideo(speakers_id) => {
                
            },
            
        };

        counter
    }
}