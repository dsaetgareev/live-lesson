use std::{collections::HashMap, cell::RefCell, rc::Rc};

use js_sys::Uint8Array;
use wasm_peers::{UserId, one_to_many::MiniServer, SessionId, ConnectionType};
use web_sys::{EncodedVideoChunk, EncodedVideoChunkInit, VideoDecoder};

use crate::{utils, utils::{inputs::{Message, ClientMessage}, device::{create_video_decoder, create_video_decoder_video, create_video_decoder_frame}, dom::create_video_id}, wrappers::EncodedVideoChunkTypeWrapper};

const TEXTAREA_ID: &str = "document-textarea";
const TEXTAREA_ID_CLIENT: &str = "client-textarea";


pub struct HostManager {
    pub players: Rc<RefCell<HashMap<UserId, String>>>,
    pub decoders: Rc<RefCell<HashMap<UserId, Rc<RefCell<VideoDecoder>>>>>,
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
        Self { 
            mini_server,
            players,
            decoders
         }
    }

    pub fn init(&mut self) {
               
        let on_open_callback = {
            let mini_server = self.mini_server.clone();
            let players = self.players.clone();
            let decoders = self.decoders.clone();
            move |user_id| {
                let text_area = match utils::dom::get_text_area(TEXTAREA_ID) {
                    Ok(text_area) => text_area,
                    Err(err) => {
                        log::error!("failed to get textarea: {:#?}", err);
                        return;
                    }
                };
                text_area.set_disabled(false);
                    text_area.set_placeholder(
                        "This is a live document shared with other users.\nWhat you write will be \
                         visible to everyone.",
                    );
                let value = text_area.value();
                log::info!("message from value {}", value.clone());
                let message = Message::Init { message: value.clone() };
                let message = serde_json::to_string(&message).unwrap();
                if !value.is_empty() {
                    mini_server
                        .send_message(user_id, &message)
                        .expect("failed to send current input to new connection");
                }
                players.borrow_mut().insert(user_id, String::default());
                let video_id = create_video_id(user_id.into_inner().to_string());
                decoders.borrow_mut().insert(user_id, Rc::new(RefCell::new(create_video_decoder_frame(video_id))));
            }
        };

        let on_message_callback = {
            let players = self.players.clone();
            let decoders = self.decoders.clone();
            move |user_id: UserId, message: String| {
                // let input = serde_json::from_str::<PlayerInput>(&message).unwrap();    
                let _ = match serde_json::from_str::<ClientMessage>(&message) {
                    Ok(input) => {
                        match input {
                            ClientMessage::ClientText { message } => {
                                let text_area = match utils::dom::get_text_area(TEXTAREA_ID_CLIENT) {
                                    Ok(text_area) => text_area,
                                    Err(err) => {
                                        log::error!("failed to get textarea: {:#?}", err);
                                        return;
                                    }
                                };
                                let client_id = text_area.get_attribute("client_id").unwrap();
                                if client_id == user_id.to_string() {
                                    text_area.set_value(&message);
                                }
                                
                                players.borrow_mut().insert(user_id, message); 
                            },
                            ClientMessage::ClientVideo { 
                                message,
                                chunk_type,
                                timestamp ,
                                duration,
                            } => {
                                let chunk_type = EncodedVideoChunkTypeWrapper::from(chunk_type.as_str()).0;
                                let video_data = Uint8Array::new_with_length(message.len().try_into().unwrap());
                                video_data.copy_from(&message);
                                let video_chunk = EncodedVideoChunkInit::new(&video_data, timestamp, chunk_type);
                                // video_chunk.duration(image.duration);
                                let chunk = EncodedVideoChunk::new(&video_chunk).unwrap();
                                

                                let mut video_vector = vec![0u8; chunk.byte_length() as usize];
                                let video_message = video_vector.as_mut();
                                chunk.copy_to_with_u8_array(video_message);
                                let data = Uint8Array::from(video_message.as_ref());
                                let mut encoded_chunk_init = EncodedVideoChunkInit::new(&data, chunk.timestamp(), chunk.type_());
                                encoded_chunk_init.duration(duration);
                                let encoded_video_chunk = EncodedVideoChunk::new(
                                    &encoded_chunk_init
                                ).unwrap();
                                let video_decoder = decoders.borrow().get(&user_id).unwrap().clone();
                                let video_decoder = video_decoder.borrow();
                                match video_decoder.state() {
                                    web_sys::CodecState::Unconfigured => {
                                        log::info!("video decoder unconfigured");
                                    },
                                    web_sys::CodecState::Configured => {
                                        video_decoder.decode(&encoded_video_chunk);
                                    },
                                    web_sys::CodecState::Closed => {
                                        log::info!("video decoder closed");
                                    },
                                    _ => {},
                                }
                            },
                        }
                    },
                    Err(err) => {
                        log::error!("failed to get input message: {:#?}", err);
                    },
                };
                
            }
        };

        self.mini_server.start(on_open_callback, on_message_callback);
    }
}