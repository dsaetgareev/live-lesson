use std::collections::HashMap;

use wasm_peers::UserId;

use super::{models::Player, host::HostGame, client::ClientGame};




pub trait Game {
    fn init(&mut self);
    fn tick(&mut self);
    fn ended(&self) -> bool;
    fn send_video_message(&mut self, video_message: &str);
    fn get_players(&self) -> HashMap<UserId, Player>;
}

pub enum FootballersGame {
    Host(HostGame),
    Client(ClientGame),
}

impl Game for FootballersGame {
    fn init(&mut self) {
        match self {
            FootballersGame::Host(game) => game.init(),
            FootballersGame::Client(game) => game.init(),
        }
    }

    fn tick(&mut self) {
        match self {
            FootballersGame::Host(game) => game.tick(),
            FootballersGame::Client(game) => game.tick(),
        }
    }

    fn ended(&self) -> bool {
        match self {
            FootballersGame::Host(game) => game.ended(),
            FootballersGame::Client(game) => game.ended(),
        }
    }

    fn send_video_message(&mut self, video_message: &str) {
        match self {
            FootballersGame::Host(game) => game.send_video_message(video_message),
            FootballersGame::Client(game) => game.send_video_message(video_message),
        }
    }

    fn get_players(&self) -> HashMap<UserId, Player> {
        match self {
            FootballersGame::Host(game) => game.get_players(),
            FootballersGame::Client(game) => game.get_players(),
        }
    }

}
