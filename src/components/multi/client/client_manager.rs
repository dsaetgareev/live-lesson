use std::{rc::Rc, cell::RefCell, sync::Arc, collections::HashMap};

use wasm_peers::{one_to_many::MiniClient, ConnectionType, SessionId, many_to_many::NetworkManager, UserId};
use crate::{models::{audio::Audio, video::Video}, utils::{ inputs::{Message, ManyMassage}, device::{create_audio_decoder, create_video_decoder_video, VideoElementKind, create_video_decoder_video_screen}, dom::{on_visible_el, create_video_id, remove_element}}, crypto::aes::Aes128State, stores::client_store::ClientMsg};

#[derive(Clone, PartialEq)]
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
            audio: Rc::new(RefCell::new(create_audio_decoder())),
            network_manager,
            audio_decoders,
            video_decoders,
        }
    }

    pub fn init(
        &mut self,
        on_action: impl Fn(ClientMsg) + 'static,
    ) {
        let on_action  = Rc::new(RefCell::new(on_action));
        let on_open_callback = {
            let on_action = on_action.clone();
            move || {
               log::error!("client manager on open");
               on_action.borrow()(ClientMsg::SendStateToHost);
            }
        };

        let video = Rc::new(RefCell::new(create_video_decoder_video("render".to_owned(), VideoElementKind::ReadyId)));
        let screen_share_decoder = create_video_decoder_video_screen("screen_share".to_owned(), VideoElementKind::ScreenBox);
        
        let on_action = on_action.clone();
        let audio = self.audio.clone();
                  
        let on_message_callback = {
            let _aes = Arc::new(Aes128State::new(true));
            let video = video.clone();
            let mut screen_share_decoder = screen_share_decoder.clone();
            let audio = audio.clone();
            move |message: Message| {
                match message {
                    Message::HostToHost { 
                        message,
                        area_kind, 
                    } => {
                        on_action.borrow()(ClientMsg::HostToHost { message, area_kind })
                    },                    
                    Message::HostToClient {
                        message,
                        area_kind
                    } => {
                        on_action.borrow()(ClientMsg::HostToClient { message, area_kind })
                    },
                    Message::InitHostArea { 
                        message,
                    } => {
                       on_action.borrow()(ClientMsg::InitHostAra(message));
                    }
                    Message::HostVideo { 
                        message,
                    } => {
                        if video.borrow().on_video {
                            let _ = video.borrow_mut().decode_break(Arc::new(message));
                        }
                    },
                    Message::HostIsScreenShare { 
                        message
                    } => {
                        on_visible_el(message, "container", "shcreen_container");
                    },
                    Message::HostScreenShare { 
                        message
                    } => {
                        let _ = screen_share_decoder.decode_break(Arc::new(message));
                    },
                    Message::HostAudio { 
                        packet
                    } => {
                        audio.borrow().decode(packet);
                    },
                    Message::HostSwitchAudio => {
                        audio.borrow_mut().on_speakers = !audio.borrow().on_speakers;
                    },
                    Message::HostSwitchVideo => {
                        video.borrow_mut().video_start = !video.borrow().video_start;
                        if video.borrow().video_start {
                            video.borrow().video_decoder.configure(&video.borrow().video_config);
                            video.borrow_mut().on_video = !video.borrow().on_video;
                            video.borrow_mut().check_key = true;
                        } else {
                            video.borrow_mut().on_video = !video.borrow().on_video;
                            video.borrow().video_decoder.reset();
                        }
                    },
                    Message::HostSwitchArea {
                        message 
                    } => {
                        on_action.borrow()(ClientMsg::HostSwitchArea(message));
                    },
                    Message::OpenPaint => {
                        on_action.borrow()(ClientMsg::OpenPaint);
                    },
                    Message::ClosePaint => {
                        remove_element("draw-canvas".to_string());
                    }
                    Message::HostPaint { 
                        offset_x,
                        offset_y,
                        action,
                    } => {
                        on_action.borrow()(ClientMsg::HostPaint { offset_x, offset_y, action });
                    },
                    Message::OnCummunication { 
                        message
                    } => {
                        on_action.borrow()(ClientMsg::OnCummunication { message })
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
                           audio.borrow().decode(packet);
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

                match video_decoders.try_borrow_mut() {
                    Ok(mut video_decoders) => {
                        match video_decoders.get(&user_id) {
                            Some(_video) => {
                                video_decoders
                                    .remove(&user_id)
                                    .expect("cannot remove video");

                                remove_element(create_video_id(user_id.to_string()));
                            },
                            None => {
                                log::error!("not found video {}", user_id.to_string());
                            },
                        }
                    },
                    Err(_) => todo!(),
                }
            }
        };
        self.network_manager.start(on_open_callback, on_message_callback, on_disconnect_callback);
    }
}
