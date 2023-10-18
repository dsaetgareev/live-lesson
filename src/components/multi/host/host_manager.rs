use std::{collections::HashMap, cell::RefCell, rc::Rc, sync::Arc};

use wasm_peers::{UserId, one_to_many::MiniServer, SessionId, ConnectionType};

use crate::{models::{client::ClientItem, video::Video, audio::Audio}, stores::host_store, utils::{dom::{create_video_id, remove_element}, device::{create_video_decoder_video, VideoElementKind, create_audio_decoder}, inputs::ClientMessage}, components::common::video};

#[derive(Clone, PartialEq)]
pub struct HostManager {
    pub players: Rc<RefCell<HashMap<UserId, ClientItem>>>,
    pub video_decoders: Rc<RefCell<HashMap<UserId, Rc<RefCell<Video>>>>>,
    pub audio_decoders: Rc<RefCell<HashMap<UserId, Rc<RefCell<Audio>>>>>,
    pub mini_server: MiniServer,
}

impl HostManager {
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
        let mini_server = MiniServer::new(signaling_server_url, session_id, connection_type)
        .expect("failed to create network manager");
        let players = Rc::new(RefCell::new(HashMap::new()));
        let video_decoders = Rc::new(RefCell::new(HashMap::new()));
        let audio_decoders = Rc::new(RefCell::new(HashMap::new()));
        Self { 
            mini_server,
            players,
            video_decoders,
            audio_decoders,
         }
    }

    pub fn init(
        &mut self,
        on_action: impl Fn(host_store::Msg) + 'static,
    ) {
        let on_action  = Rc::new(RefCell::new(on_action));
        let on_open_callback = {
            let on_action = on_action.clone();
            let video_decoders = self.video_decoders.clone();
            let audio_decoders = self.audio_decoders.clone();
            move |user_id: UserId| {
                let video_id = create_video_id(user_id.into_inner().to_string());
                video_decoders.borrow_mut()
                    .insert(
                        user_id,
                        Rc::new(RefCell::new(create_video_decoder_video(video_id, VideoElementKind::HostBox)))
                    );
                audio_decoders.borrow_mut()
                    .insert(user_id, Rc::new(RefCell::new(create_audio_decoder())));
                on_action.borrow()(host_store::Msg::AddClient(user_id));
            }
        };

        let on_message_callback = {
            let on_action = on_action.clone();
            let video_decoders = self.video_decoders.clone();
            let audio_decoders = self.audio_decoders.clone();
            move |user_id: UserId, message: ClientMessage| { 
                match message {
                    ClientMessage::InitClient { 
                        message
                    } => {
                        on_action.borrow()(host_store::Msg::InitClient(user_id, message));
                    }
                    ClientMessage::ClientText { message: _ } => {
                        // on_tick.borrow()();
                    },
                    ClientMessage::ClientVideo { 
                        message,
                    } => {
                        let video = video_decoders.as_ref().borrow().get(&user_id).unwrap().clone();
                        match video.clone().as_ref().try_borrow_mut() {
                            Ok(mut video) => {
                                let _ = video.decode_break(Arc::new(message));
                            },
                            Err(_) => {
                                
                            },
                        }
                    },
                    ClientMessage::ClientAudio { 
                        packet
                    } => {
                        let audio = audio_decoders.as_ref().borrow().get(&user_id).unwrap().clone();
                        audio.borrow().decode(packet);
                    }
                    ClientMessage::ClientSwitchVideo { 
                        message
                    } => {
                        on_action.borrow()(host_store::Msg::ClientSwitchVideo(user_id, message));
                    },
                    ClientMessage::ClientToClient { 
                        message,
                        area_kind
                    } => {
                       on_action.borrow()(host_store::Msg::ClientToClient(user_id, message, area_kind));                                                
                    },
                    ClientMessage::ClientSwitchArea { 
                        message
                    } => {
                        on_action.borrow()(host_store::Msg::ClientSwitchArea(user_id, message));
                    }
                }            
            }
        };

        let on_disconnect_callback = {
            let on_action = on_action.clone();
            let video_decoders = self.video_decoders.clone();
            let audio_decoders = self.audio_decoders.clone();
            move |user_id: UserId| {
                log::error!("disconected {}", user_id);
                
                match video_decoders.try_borrow_mut() {
                    Ok(mut video_decoders) => {
                        match video_decoders.get(&user_id) {
                            Some(_video) => {
                                video_decoders
                                    .remove(&user_id)
                                    .expect("cannot remove video");
                            },
                            None => {
                                log::error!("not found video {}", user_id.to_string());
                            },
                        }
                    },
                    Err(_) => todo!(),
                }

                match audio_decoders.try_borrow_mut() {
                    Ok(mut audio_decoders) => {
                        match audio_decoders.get(&user_id) {
                            Some(_audio) => {
                                audio_decoders
                                    .remove(&user_id)
                                    .expect("cannot remove video");
                            },
                            None => {
                                log::error!("not found video {}", user_id.to_string());
                            },
                        }
                    },
                    Err(_) => todo!(),
                }

                on_action.borrow()(host_store::Msg::DisconnectClient(user_id));
            }
        };
        self.mini_server.start(on_open_callback, on_message_callback, on_disconnect_callback);
    }
}