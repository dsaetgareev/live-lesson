use std::{collections::HashMap, cell::RefCell, rc::Rc, sync::Arc};

use wasm_bindgen::prelude::Closure;
use wasm_peers::{UserId, one_to_many::MiniServer, SessionId, ConnectionType};
use web_sys::{ EncodedAudioChunkInit, EncodedAudioChunk };

use crate::{utils, utils::{inputs::{Message, ClientMessage}, dom::create_video_id, device::{create_video_decoder_frame, create_audio_decoder}}, models::video::Video, wrappers::EncodedAudioChunkTypeWrapper};


const TEXTAREA_ID: &str = "document-textarea";
const TEXTAREA_ID_CLIENT: &str = "client-textarea";


pub struct HostManager {
    pub players: Rc<RefCell<HashMap<UserId, String>>>,
    pub decoders: Rc<RefCell<HashMap<UserId, Rc<RefCell<Video>>>>>,
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
            decoders,
         }
    }

    pub fn init(
        &mut self,
        on_tick: impl Fn() + 'static,
    ) {
        let on_tick  = Rc::new(RefCell::new(on_tick));
        let on_open_callback = {
            let mini_server = self.mini_server.clone();
            let players = self.players.clone();
            let decoders = self.decoders.clone();
            let on_tick = on_tick.clone();
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
                players.as_ref().borrow_mut().insert(user_id, String::default());
                let video_id = create_video_id(user_id.into_inner().to_string());
                on_tick.borrow()();
                decoders.as_ref().borrow_mut().insert(user_id, Rc::new(RefCell::new(create_video_decoder_frame(video_id))));
                on_tick.borrow()();
            }
        };

        let on_message_callback = {
            let players = self.players.clone();
            let decoders = self.decoders.clone();
            let audio = create_audio_decoder();
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
                                
                                players.as_ref().borrow_mut().insert(user_id, message);
                            },
                            ClientMessage::ClientVideo { 
                                message,
                            } => {
                                let video = decoders.as_ref().borrow().get(&user_id).unwrap().clone();
                                let mut video = video.as_ref().borrow_mut();
                                let _ = video.decode_break(Arc::new(message));
                            },
                            ClientMessage::ClientAudio { 
                                message,
                                chunk_type,
                                timestamp,
                                duration
                            } => {
                                let _ = audio.audio_context.resume();
                                    let chunk_type = EncodedAudioChunkTypeWrapper::from(chunk_type).0;
                                    let audio_data = &message;
                                    let audio_data_js: js_sys::Uint8Array =
                                        js_sys::Uint8Array::new_with_length(audio_data.len() as u32);
                                    audio_data_js.copy_from(audio_data.as_slice());
                                    let chunk_type = EncodedAudioChunkTypeWrapper(chunk_type);
                                    let mut audio_chunk_init =
                                        EncodedAudioChunkInit::new(&audio_data_js.into(), timestamp, chunk_type.0);
                                    audio_chunk_init.duration(duration);
                                    let encoded_audio_chunk = EncodedAudioChunk::new(&audio_chunk_init).unwrap();
    
                                    match audio.audio_decoder.state() {
                                        web_sys::CodecState::Unconfigured => {
                                            log::info!("audio decoder unconfigured");
                                        },
                                        web_sys::CodecState::Configured => {
                                            audio.audio_decoder.decode(&encoded_audio_chunk);
                                        },
                                        web_sys::CodecState::Closed => {
                                            log::info!("audio_decoder closed");
                                        },
                                        _ => {}
                                    }    
                            }
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