use std::{collections::HashMap, cell::RefCell, rc::Rc, sync::Arc};

use wasm_peers::{UserId, one_to_many::MiniServer, SessionId, ConnectionType};

use crate::{ utils::{inputs::{Message, ClientMessage}, dom::{create_video_id, on_visible_el, remove_element}, device::{ create_audio_decoder, create_video_decoder_video, VideoElementKind }}, models::{video::Video, client::{ClientProps, ClientItem}, host::HostPorps, commons::AreaKind, audio::Audio, packet::AudioPacket}, stores::host_store};

#[derive(Clone, PartialEq)]
pub struct HostManager {
    pub players: Rc<RefCell<HashMap<UserId, ClientItem>>>,
    pub decoders: Rc<RefCell<HashMap<UserId, Rc<RefCell<Video>>>>>,
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
        let decoders = Rc::new(RefCell::new(HashMap::new()));
        let audio_decoders = Rc::new(RefCell::new(HashMap::new()));
        Self { 
            mini_server,
            players,
            decoders,
            audio_decoders,
         }
    }

    pub fn init(
        &mut self,
        on_action: impl Fn(host_store::Msg) + 'static,
        host_props: Rc<RefCell<HostPorps>>,
        client_props: Rc<RefCell<ClientProps>>,
    ) {
        let on_action  = Rc::new(RefCell::new(on_action));
        let on_open_callback = {
            let on_action = on_action.clone();
            move |user_id: UserId| {
                on_action.borrow()(host_store::Msg::AddClient(user_id));
            }
        };

        let on_message_callback = {
            let players = self.players.clone();
            let decoders = self.decoders.clone();
            let audio_decoders = self.audio_decoders.clone();
            let on_tick = on_action.clone();
            move |user_id: UserId, message: ClientMessage| { 
                match message {
                    ClientMessage::ClientText { message: _ } => {
                        // on_tick.borrow()();
                    },
                    ClientMessage::ClientVideo { 
                        message,
                    } => {
                        let video = decoders.as_ref().borrow().get(&user_id).unwrap().clone();
                        let mut video = video.as_ref().borrow_mut();
                        let _ = video.decode_break(Arc::new(message));
                    },
                    ClientMessage::ClientAudio { 
                        packet
                    } => {
                        let audio = audio_decoders.as_ref().borrow().get(&user_id).unwrap().clone();
                        
                        let encoded_audio_chunk = AudioPacket::get_encoded_audio_chunk(packet);
                        let state = audio.borrow().audio_decoder.state();
                        match state {
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
                    ClientMessage::ClientSwitchVideo { 
                        message
                    } => {
                        let video_id = create_video_id(user_id.to_string());
                        let client_logo_id = create_video_id(format!("{}_{}", "client-video-logo", user_id.to_string()));
                        on_visible_el(message, &video_id, &client_logo_id);
                        // on_tick.borrow()();
                    },
                    ClientMessage::ClientToClient { 
                        message,
                        area_kind
                    } => {
                       
                        match players.as_ref().borrow_mut().get_mut(&user_id) {
                            Some(client_item) => {
                                client_item.set_area_kind(area_kind);
                                match area_kind {
                                    AreaKind::Editor => {
                                        client_item.set_editor_content(message.clone());
                                        if client_props.borrow().client_id == user_id.to_string() {
                                            client_props.borrow_mut().set_editor_content(message);
                                            client_props.borrow_mut().is_write = true;
                                        }
                                    },
                                    AreaKind::TextArea => {
                                        client_item.set_text_area_content(message.clone());
                                        if client_props.borrow().client_id == user_id.to_string() {
                                            client_props.borrow_mut().set_text_area_content(message);
                                        }
                                    },
                                }
                                
                            },
                            None => {
                                log::error!("cannot find client item, id: {}", user_id.to_string());
                            },
                        }
                        
                        // on_tick.borrow()();
                                                
                    },
                    ClientMessage::ClientSwitchArea { 
                        message
                    } => {
                        if client_props.borrow().client_id == user_id.to_string() {
                            client_props.borrow_mut().set_area_kind(message);
                        }

                        match players.as_ref().borrow_mut().get_mut(&user_id) {
                            Some(client_item) => {
                                client_item.set_area_kind(message)
                            },
                            None => todo!(),
                        }
                        
                        // on_tick.borrow()();
                    }
                }            
            }
        };

        let on_disconnect_callback = {
            let players = self.players.clone();
            let on_tick = on_action.clone();
            move |user_id: UserId| {
                let box_id = format!("item-box-{}", user_id.clone());
                players.borrow_mut()
                    .remove(&user_id)
                    .expect("cannot remove user");
                remove_element(box_id);
                // on_tick.borrow()();
            }
        };
        self.mini_server.start(on_open_callback, on_message_callback, on_disconnect_callback);
    }
}