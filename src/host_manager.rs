use std::{collections::HashMap, cell::RefCell, rc::Rc};

use wasm_peers::{UserId, one_to_many::MiniServer, SessionId, ConnectionType};

use crate::{utils, inputs::Message};

const TEXTAREA_ID: &str = "document-textarea";
const TEXTAREA_ID_CLIENT: &str = "client-textarea";


pub struct HostManager {
    pub players: Rc<RefCell<HashMap<UserId, String>>>,
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
        Self { 
            mini_server,
            players: players,
         }
    }

    pub fn init(&mut self) {
               
        let on_open_callback = {
            let mini_server = self.mini_server.clone();
            let players = self.players.clone();
            move |user_id| {
                let text_area = match utils::get_text_area(TEXTAREA_ID) {
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
            }
        };

        let on_message_callback = {
            let players = self.players.clone();
            move |user_id: UserId, message: String| {
                // let input = serde_json::from_str::<PlayerInput>(&message).unwrap();
                log::info!("input {}", message);
                log::info!("user_id {}", user_id);           
                
                let text_area = match utils::get_text_area(TEXTAREA_ID_CLIENT) {
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
            }
        };

        self.mini_server.start(on_open_callback, on_message_callback);
    }
}