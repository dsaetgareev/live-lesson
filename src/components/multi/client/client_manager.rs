use std::{rc::Rc, cell::RefCell, sync::Arc, collections::HashMap};

use wasm_bindgen::JsCast;
use wasm_peers::{one_to_many::MiniClient, ConnectionType, SessionId, many_to_many::NetworkManager, UserId};
use web_sys::HtmlCanvasElement;
use yew::Callback;

use crate::{models::{host::HostPorps, client::ClientProps, commons::AreaKind, audio::Audio, video::Video, packet::AudioPacket}, utils::{ inputs::{Message, PaintAction, ManyMassage}, device::{create_audio_decoder, create_video_decoder_video, VideoElementKind}, dom::{on_visible_el, create_video_id, switch_visible_el, remove_element}}, crypto::aes::Aes128State, components::multi::draw::paint};


pub struct ClientManager {
    pub mini_client: MiniClient,
    pub audio: Rc<RefCell<Audio>>,
    pub network_manager: NetworkManager,
    pub audio_decoders: Rc<RefCell<HashMap<UserId, Rc<RefCell<Audio>>>>>,
    pub video_decoders: Rc<RefCell<HashMap<UserId, Rc<RefCell<Video>>>>>,
}

impl ClientManager {
    pub fn new(
        session_id: SessionId,
        audio: Rc<RefCell<Audio>>
    ) -> Self {
        let connection_type = ConnectionType::StunAndTurn {
            stun_urls: env!("STUN_SERVER_URLS").to_string(),
            turn_urls: env!("TURN_SERVER_URLS").to_string(),
            username: env!("TURN_SERVER_USERNAME").to_string(),
            credential: env!("TURN_SERVER_CREDENTIAL").to_string(),
        };
        let signaling_server_url = concat!(env!("SIGNALING_SERVER_URL"), "/one-to-many");
        let mini_client = MiniClient::new(signaling_server_url, session_id, connection_type.clone())
            .expect("failed to create mini client");
        let network_manager = NetworkManager::new(
            concat!(env!("SIGNALING_SERVER_URL"), "/many-to-many"),
            session_id.clone(),
            connection_type,
        )
        .expect("failed to create network manager");
        let audio_decoders = Rc::new(RefCell::new(HashMap::new()));
        let video_decoders = Rc::new(RefCell::new(HashMap::new()));
        Self { 
            mini_client,
            audio,
            network_manager,
            audio_decoders,
            video_decoders
        }
    }

    pub fn init(
        &mut self,
        on_tick: impl Fn() + 'static,
        host_props: Rc<RefCell<HostPorps>>,
        client_props: Rc<RefCell<ClientProps>>,
    ) {
        let on_tick  = Rc::new(RefCell::new(on_tick));
        let on_open_callback = {
            move || {
               log::error!("client manager on open");
            }
        };

        let video = create_video_decoder_video("render".to_owned(), VideoElementKind::ReadyId);
        let screen_share_decoder = create_video_decoder_video("screen_share".to_owned(), VideoElementKind::ReadyId);
        
        let on_tick = on_tick.clone();
        let mut paints: HashMap<i32, Rc<HtmlCanvasElement>> = HashMap::new();
        let audio = self.audio.clone();
        let on_message_callback = {
            let _aes = Arc::new(Aes128State::new(true));
            let mut video = video.clone();
            let mut screen_share_decoder = screen_share_decoder.clone();
            let audio = audio.clone();
            let host_props = host_props.clone();
            move |message: Message| {
                match message {
                    Message::HostToHost { 
                        message,
                        area_kind, 
                    } => {
                        match area_kind {
                            AreaKind::Editor => {
                                host_props.borrow_mut().set_editor_content(message);
                            },
                            AreaKind::TextArea => {
                                host_props.borrow_mut().host_area_content.set_content(message)
                            },
                        }
                        
                        on_tick.borrow()();
                    },
                    Message::HostToClient {
                        message,
                        area_kind
                    } => {
                        log::error!("message {}", message);
                        match area_kind {
                            AreaKind::Editor => {
                                client_props.borrow_mut().set_editor_content(message);
                            },
                            AreaKind::TextArea => {
                                client_props.borrow_mut().set_text_area_content(message);
                            },
                        }                                
                        on_tick.borrow()();
                    },
                    Message::Init { 
                        editor_content,
                        text_area_content,
                        area_kind,
                        is_communication
                    } => {
                        host_props.borrow_mut().host_area_content.set_content(text_area_content);
                        host_props.borrow_mut().set_editor_content(editor_content);
                        host_props.borrow_mut().set_host_area_kind(area_kind);
                        host_props.borrow_mut().set_communication(is_communication);
                        on_tick.borrow()();
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
                                    if let Err(err) = video.decode_break(Arc::new(message)) {
                                        log::error!("error on decode {}", err);
                                    }
                                },
                                web_sys::CodecState::Closed => {
                                    log::info!("video decoder closed");
                                    video = create_video_decoder_video("render".to_owned(), VideoElementKind::ReadyId);
                                    video.check_key = true;
                                },
                                _ => {},
                            }
                        }
                    },
                    Message::HostIsScreenShare { 
                        message
                    } => {
                        on_visible_el(message, "container", "shcreen_container");
                        on_tick.borrow()();
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
                        packet
                    } => {
                        if audio.borrow().on_speakers {
                            
                            let encoded_audio_chunk = AudioPacket::get_encoded_audio_chunk(packet);

                            match audio.borrow().audio_decoder.state() {
                                web_sys::CodecState::Unconfigured => {
                                    log::info!("audio decoder unconfigured");
                                },
                                web_sys::CodecState::Configured => {
                                    audio.borrow().audio_decoder.decode(&encoded_audio_chunk);
                                },
                                web_sys::CodecState::Closed => {
                                    log::info!("audio_decoder closed");
                                },
                                _ => {}
                            }    
                        }
                        
                    },
                    Message::HostSwitchAudio => {
                        audio.borrow_mut().on_speakers = !audio.borrow().on_speakers;
                    },
                    Message::HostSwitchVideo => {
                        video.video_start = !video.video_start;
                        if video.video_start {
                            video.video_decoder.configure(&video.video_config);
                            video.on_video = !video.on_video;
                            video.check_key = true;
                        } else {
                            video.on_video = !video.on_video;
                            video.video_decoder.reset();
                        }
                    },
                    Message::HostSwitchArea {
                        message 
                    } => {
                        host_props.borrow_mut().set_host_area_kind(message);
                        on_tick.borrow()();
                    },
                    Message::OpenPaint => {
                        match host_props.borrow().host_area_kind {
                            AreaKind::Editor => {
                                let canvas = paint::start(&host_props.borrow().host_editor_content, Callback::default())
                                    .expect("cannot get canvas");
                                paints.insert(1, canvas);
                            },
                            AreaKind::TextArea => {
                                let canvas = paint::start(&host_props.borrow().host_area_content.content, Callback::default())
                                    .expect("cannot get canvas");
                                paints.insert(1, canvas);
                            },
                        }
                        on_tick.borrow()();
                    },
                    Message::HostPaint { 
                        offset_x,
                        offset_y,
                        action,
                    } => {
                        let key = 1;
                        let canvas = paints.get(&key).expect("cannot get canvas");
                        let context = canvas
                            .get_context("2d")
                            .expect("cannot get canvas")
                            .unwrap()
                            .dyn_into::<web_sys::CanvasRenderingContext2d>()
                            .expect("cannot get canvas");
                        match action {
                            PaintAction::Down => {
                                context.begin_path();
                                context.move_to(offset_x, offset_y);
                            },
                            PaintAction::Move => {
                                context.line_to(offset_x, offset_y);
                                context.stroke();
                                context.begin_path();
                                context.move_to(offset_x, offset_y);
                            },
                            PaintAction::Up => {
                                context.line_to(offset_x, offset_y);
                                context.stroke();
                            },
                        };
                        

                    },
                    Message::OnCummunication { 
                        message
                    } => {
                        switch_visible_el(message, "video-box");
                        host_props.borrow_mut().set_communication(message);
                    }
                }
            } 
        
        };

        let on_disconnect_callback = {
            move |_user_id: UserId| {
                
            }
        };
        self.mini_client.start(on_open_callback, on_message_callback, on_disconnect_callback);
    }

    pub fn many_init(&mut self) {
        let audio_decoders = self.audio_decoders.clone();
        let video_decoders = self.video_decoders.clone();
        
        let on_open_callback = {
            
            move |user_id: UserId| {
                audio_decoders.as_ref().borrow_mut().insert(user_id, Rc::new(RefCell::new(create_audio_decoder())));
                let video_id = create_video_id(user_id.into_inner().to_string());
                video_decoders.as_ref().borrow_mut().insert(user_id, Rc::new(RefCell::new(create_video_decoder_video(video_id, VideoElementKind::ClentBox))));                
            }
        };

        let on_message_callback = {
            let _aes = Arc::new(Aes128State::new(true));
            let audio_decoders = self.audio_decoders.clone();
            let video_decoders = self.video_decoders.clone();
            move |user_id: UserId, message: ManyMassage| {
                match message {
                    ManyMassage::Audio { 
                        packet
                    } => {
                        let audio = audio_decoders.as_ref().borrow().get(&user_id).unwrap().clone();
                        if audio.borrow().on_speakers {
                            
                            let encoded_audio_chunk = AudioPacket::get_encoded_audio_chunk(packet);

                            match audio.borrow().audio_decoder.state() {
                                web_sys::CodecState::Unconfigured => {
                                    log::info!("audio decoder unconfigured");
                                },
                                web_sys::CodecState::Configured => {
                                    audio.borrow().audio_decoder.decode(&encoded_audio_chunk);
                                },
                                web_sys::CodecState::Closed => {
                                    log::info!("audio_decoder closed");
                                },
                                _ => {}
                            }    
                        }
                    },
                    ManyMassage::Video { 
                        packet
                    } => {
                        let video = video_decoders.as_ref().borrow().get(&user_id).unwrap().clone();
                        let mut video = video.as_ref().borrow_mut();
                        let _ = video.decode_break(Arc::new(packet));
                    }
                }
            } 
        
        };
        let on_disconnect_callback = {
            let video_decoders = self.video_decoders.clone();
            move |user_id: UserId| {
                video_decoders
                    .borrow_mut()
                    .remove(&user_id)
                    .expect("cannot remove decoders");
                remove_element(create_video_id(user_id.to_string()));
            }
        };
        self.network_manager.start(on_open_callback, on_message_callback, on_disconnect_callback);
    }
}
