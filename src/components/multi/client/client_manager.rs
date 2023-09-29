use std::{rc::Rc, cell::RefCell, sync::Arc};

use wasm_peers::{one_to_many::MiniClient, ConnectionType, SessionId};
use web_sys::{EncodedAudioChunkInit, EncodedAudioChunk};

use crate::{models::{host::HostPorps, client::ClientProps, commons::AreaKind}, utils::{ inputs::Message, device::{create_audio_decoder, create_video_decoder_video}, dom::on_visible_el}, crypto::aes::Aes128State, wrappers::EncodedAudioChunkTypeWrapper};


pub struct ClientManager {
    pub mini_client: MiniClient
}

impl ClientManager {
    pub fn new(session_id: SessionId) -> Self {
        let connection_type = ConnectionType::StunAndTurn {
            stun_urls: env!("STUN_SERVER_URLS").to_string(),
            turn_urls: env!("TURN_SERVER_URLS").to_string(),
            username: env!("TURN_SERVER_USERNAME").to_string(),
            credential: env!("TURN_SERVER_CREDENTIAL").to_string(),
        };
        let signaling_server_url = concat!(env!("SIGNALING_SERVER_URL"), "/one-to-many");
        let mini_client = MiniClient::new(signaling_server_url, session_id, connection_type)
            .expect("failed to create network manager");
        Self { 
            mini_client
        }
    }

    pub fn init(
        &mut self,
        on_tick: impl Fn() + 'static,
        host_props: Rc<RefCell<HostPorps>>,
        client_props: Rc<RefCell<ClientProps>>,
    ) {
        let is_ready = Rc::new(RefCell::new(false));
        let on_tick  = Rc::new(RefCell::new(on_tick));
        let on_open_callback = {
            let mini_client = self.mini_client.clone();
            let is_ready = Rc::clone(&is_ready);
            let host_props = host_props.clone();
            let on_tick = on_tick.clone();
            move || {
                if !*is_ready.borrow() {
                    host_props.borrow_mut().host_area_content.is_disabled = false;
                    host_props.borrow_mut().host_area_content.set_placeholder(
                        "This is a live document shared with other users.\nWhat you write will be \
                         visible to everyone.".to_string()
                    );
                    *is_ready.borrow_mut() = true;
                }
                let editor_content = &host_props.borrow().host_editor_content;
                let text_area_content = &host_props.borrow().host_area_content.content;
                let message = Message::Init { 
                    editor_content: editor_content.clone(),
                    text_area_content: text_area_content.clone(),
                    area_kind: host_props.borrow().host_area_kind
                };
                let message = serde_json::to_string(&message).unwrap();
                if !editor_content.is_empty() || !text_area_content.is_empty() {
                    mini_client
                        .send_message_to_host(&message)
                        .expect("failed to send current input to new connection");
                }
                on_tick.borrow()();
            }
        };

        let video = create_video_decoder_video("render".to_owned());
        let screen_share_decoder = create_video_decoder_video("screen_share".to_owned());
        let audio = create_audio_decoder();
        let on_tick = on_tick.clone();
        let on_message_callback = {
            let _aes = Arc::new(Aes128State::new(true));
            let mut video = video.clone();
            let mut screen_share_decoder = screen_share_decoder.clone();
            let mut audio = audio.clone();
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
                        area_kind
                    } => {
                        host_props.borrow_mut().host_area_content.set_content(text_area_content);
                        host_props.borrow_mut().set_editor_content(editor_content);
                        host_props.borrow_mut().set_host_area_kind(area_kind);
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
                                    if let Err(err) = video.decode(Arc::new(message)) {
                                        log::error!("error on decode {}", err);
                                    }
                                },
                                web_sys::CodecState::Closed => {
                                    log::info!("video decoder closed");
                                    video.video_decoder.configure(&video.video_config);
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
                        message,
                        chunk_type,
                        timestamp,
                        duration
                    } => {     
                        if audio.on_speakers {
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
                        
                    },
                    Message::HostSwitchAudio => {
                        audio.on_speakers = !audio.on_speakers;
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
                    }
                }
            } 
        
        };
        
        self.mini_client.start(on_open_callback, on_message_callback);
    }
}
