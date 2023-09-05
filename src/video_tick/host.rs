use std::{collections::HashMap, rc::Rc, cell::RefCell};

use wasm_peers::{UserId, SessionId, ConnectionType, one_to_many::MiniServer};

use crate::utils;

use super::{models::{Player, PlayerInput, Message}, footballers_game::Game};


const TEXTAREA_ID: &str = "host-textarea";
const TEXTAREA_ID_CLIENT: &str = "client-textarea";


pub struct HostGameInner {
    mini_server: MiniServer,
    host_player: Option<Player>,
    players: HashMap<UserId, Player>,
    player_input: Rc<RefCell<PlayerInput>>,
    game_started: bool,
}

impl HostGameInner {
    pub(self) fn new(
        session_id: SessionId,
        connection_type: ConnectionType,
        signaling_server_url: &str,
    ) -> HostGameInner {
        let mini_server = MiniServer::new(signaling_server_url, session_id, connection_type)
        .expect("failed to create network manager");
        Self { 
            mini_server,
            host_player: None,
            players: HashMap::new(),
            player_input: local_player_input(),
            game_started: false,
         }
    }

    pub(self) fn tick(&mut self) {

        let text_area = utils::get_text_area(TEXTAREA_ID).unwrap();
        let value = text_area.value();
        self.player_input.borrow_mut().value = value.clone();

        let client_text_area = utils::get_text_area(TEXTAREA_ID_CLIENT).unwrap();
        let client_value = client_text_area.value();

        self.host_player
            .as_mut()
            .unwrap()
            .set_input(self.player_input.borrow().clone());


        let message = Message::GameState { 
            players: self.get_player_entities(),
            host_value: value,
            client_value,
         };

        let game_state = serde_json::to_string(&message).unwrap();

        self.mini_server.send_message_to_all(&game_state);
    }

    fn get_player_entities(&self) -> Vec<Player> {
        let mut v: Vec<Player> = self
            .players
            .values()
            .map(|player| Player { number: player.number, current_input: PlayerInput { value: player.get_input().value } })
            .collect();

        v.push(
            self.host_player.clone()
                .unwrap(),
        );
        v
    }
}


pub struct HostGame {
    inner: Rc<RefCell<HostGameInner>>,
}

impl HostGame {
    pub fn new(
        session_id: SessionId,
        connection_type: ConnectionType,
        signaling_server_url: &str,
    ) -> HostGame {
        HostGame {
            inner: Rc::new(RefCell::new(HostGameInner::new(
                session_id,
                connection_type,
                signaling_server_url,
            ))),
        }
    }
}

impl Game for HostGame {
    fn init(&mut self) {
        let host_player = Player::new(1);
        self.inner.borrow_mut().host_player = Some(host_player);
        let host_game = self.inner.clone();
        let number = *(&self.inner.borrow().players.len());

        let on_open_callback = move |user_id| {
            let game_state = Message::GameInit {
                players: host_game.borrow().get_player_entities(),
            };
            let game_state = serde_json::to_string(&game_state).unwrap();
            let _ = host_game
                .borrow()
                .mini_server
                .send_message(user_id, &game_state);
            host_game.borrow_mut().game_started = true;
            let player = Player::new(number);
            host_game.borrow_mut().players.insert(user_id, player);
        };

        let host_game = self.inner.clone();
        let on_message_callback = move |user_id, message: String| {
            let input = serde_json::from_str::<PlayerInput>(&message).unwrap();
            log::info!("input {}", input.value);
            host_game
                .borrow_mut()
                .players
                .get_mut(&user_id)
                .expect("no player instance for this user_id")
                .set_input(input);
        };

        self.inner
            .borrow_mut()
            .mini_server
            .start(on_open_callback, on_message_callback);
    }

    fn tick(&mut self) {
        self.inner.borrow_mut().tick();
    }

    fn ended(&self) -> bool {
        false
    }

    fn send_video_message(&mut self, _video_message: &str) {
        todo!()
    }

    fn get_players(&self) -> HashMap<UserId, Player> {
        self.inner.borrow().players.clone()
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