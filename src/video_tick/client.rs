use std::{rc::Rc, cell::RefCell, collections::HashMap};

use wasm_peers::{SessionId, ConnectionType, one_to_many::MiniClient, UserId};

use crate::utils;

use super::{models::{Player, PlayerInput, Message}, footballers_game::Game};


const TEXTAREA_ID: &str = "host-textarea";
const TEXTAREA_ID_CLIENT: &str = "client-textarea";


pub struct ClientGameInner {
    mini_client: MiniClient,
    players: Vec<Player>,
    player_input: Rc<RefCell<PlayerInput>>,
}

impl ClientGameInner {
    pub(self) fn new(
        session_id: SessionId,
        connection_type: ConnectionType,
        signaling_server_url: &str,
    ) -> Self {
        let mini_client = MiniClient::new(signaling_server_url, session_id, connection_type)
        .expect("failed to create network manager");
        
        ClientGameInner {
            mini_client,
            players: Vec::new(),
            player_input: local_player_input(),
        }
    }

    fn tick(&mut self) {

        let text_area = utils::get_text_area(TEXTAREA_ID_CLIENT).unwrap();
        let value = text_area.value();
        log::info!("qqqqqqqqqqqqqqqqqqqqqqqqqqq {}", value.clone());
        self.player_input.borrow_mut().value = value.clone();

        // on each frame, send input to host
        log::info!("qqqqqqqqqqqqqqqqqqqqqqqqqqq {}", &self.player_input.borrow().value);
        let message = serde_json::to_string::<PlayerInput>(&self.player_input.borrow()).unwrap();



        // allow some messages to fail
        let _ = self.mini_client.send_message_to_host(&message);

    }
}

pub struct ClientGame {
    inner: Rc<RefCell<ClientGameInner>>,
}

impl ClientGame {
    pub fn new(
        session_id: SessionId,
        connection_type: ConnectionType,
        signaling_server_url: &str,
    ) -> Self {
        ClientGame {
            inner: Rc::new(RefCell::new(ClientGameInner::new(
                session_id,
                connection_type,
                signaling_server_url,
            ))),
        }
    }
}

impl Game for ClientGame {
    fn init(&mut self) {
        let on_open_callback = || {};

        let inner = self.inner.clone();

        let on_message_callback = move |message: String| {
            let message = serde_json::from_str::<Message>(&message).unwrap();

            match message {
                Message::GameInit {
                    players,
                } => {
                    inner.borrow_mut().players = players;
                }
                Message::GameState { players, host_value, client_value} => {
                    inner.borrow_mut().players = players;
                    let text_area = utils::get_text_area(TEXTAREA_ID).unwrap();
                    text_area.set_value(&host_value);

                    let client_text_area = utils::get_text_area(TEXTAREA_ID_CLIENT).unwrap();
                    client_text_area.set_value(&client_value);
                }
                Message::GoalScored => todo!(),
                Message::GameEnded => todo!(),
            }
        };

        self.inner
            .borrow_mut()
            .mini_client
            .start(on_open_callback, on_message_callback);
    }

    fn tick(&mut self) {
        self.inner.borrow_mut().tick();
    }

    fn ended(&self) -> bool {
        todo!()
    }

    fn send_video_message(&mut self, _video_message: &str) {
        todo!()
    }

    fn get_players(&self) -> HashMap<UserId, Player> {
        let players_map: HashMap<UserId, Player> = HashMap::default();
        players_map
    }

}

fn local_player_input() -> Rc<RefCell<PlayerInput>> {
    let player_input = Rc::new(RefCell::new(PlayerInput::default()));
    let text_area = match utils::get_text_area(TEXTAREA_ID) {
        Ok(text_area) => text_area,
        Err(err) => {
            log::error!("failed to get textarea: {:#?}", err);
            return player_input;
        }
    };
    player_input.borrow_mut().value = text_area.value();
    player_input
}