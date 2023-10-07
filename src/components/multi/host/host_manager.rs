use std::{collections::HashMap, cell::RefCell, rc::Rc, sync::Arc};

use wasm_peers::{UserId, one_to_many::MiniServer, SessionId, ConnectionType};
use web_sys::{ EncodedAudioChunkInit, EncodedAudioChunk };

use crate::{ utils::{inputs::{Message, ClientMessage}, dom::{create_video_id, on_visible_el}, device::{create_video_decoder_frame, create_audio_decoder, create_video_decoder_video }}, models::{video::Video, client::{ClientProps, ClientItem}, host::HostPorps, commons::AreaKind, audio::Audio, packet::AudioPacket}, wrappers::EncodedAudioChunkTypeWrapper};


pub struct HostManager {
    pub players: Rc<RefCell<HashMap<UserId, ClientItem>>>,
    pub decoders: Rc<RefCell<HashMap<UserId, Rc<RefCell<Video>>>>>,
    pub audio_decoders: Rc<RefCell<HashMap<UserId, Rc<RefCell<Audio>>>>>,
    pub mini_server: MiniServer,
}

impl HostManager {
    pub fn new(
        session_id: SessionId,
        connection_type: ConnectionType,
        signaling_server_url: &str,
    ) -> Self {
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
        on_tick: impl Fn() + 'static,
        host_props: Rc<RefCell<HostPorps>>,
        client_props: Rc<RefCell<ClientProps>>,
    ) {
        let on_tick  = Rc::new(RefCell::new(on_tick));
        let on_open_callback = {
            let mini_server = self.mini_server.clone();
            let players = self.players.clone();
            let decoders = self.decoders.clone();
            let audio_decoders = self.audio_decoders.clone();
            let on_tick = on_tick.clone();
            move |user_id: UserId| {
                let editor_content = &host_props.borrow().host_editor_content;
                let text_area_content = &host_props.borrow().host_area_content.content;
                let area_kind = host_props.borrow().host_area_kind;
                let is_communication = host_props.borrow().is_communication;
                let message = Message::Init { 
                    editor_content: editor_content.clone(),
                    text_area_content: text_area_content.clone(),
                    area_kind: area_kind.clone(),
                    is_communication
                };
                mini_server
                    .send_message(user_id, &message)
                    .expect("failed to send current input to new connection");
                players.as_ref().borrow_mut().insert(user_id, ClientItem::new(area_kind));
                let video_id = create_video_id(user_id.into_inner().to_string());
                decoders.as_ref().borrow_mut().insert(user_id, Rc::new(RefCell::new(create_video_decoder_frame(video_id))));
                audio_decoders.as_ref().borrow_mut().insert(user_id, Rc::new(RefCell::new(create_audio_decoder())));
                on_tick.borrow()();
            }
        };

        let on_message_callback = {
            let players = self.players.clone();
            let decoders = self.decoders.clone();
            let audio_decoders = self.audio_decoders.clone();
            let on_tick = on_tick.clone();
            move |user_id: UserId, message: ClientMessage| { 
                match message {
                    ClientMessage::ClientText { message: _ } => {
                        on_tick.borrow()();
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
                        on_tick.borrow()();
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
                        
                        on_tick.borrow()();
                                                
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
                        
                        on_tick.borrow()();
                    }
                }            
            }
        };

        self.mini_server.start(on_open_callback, on_message_callback);
    }
}