use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize)]
pub enum Message {
    GameInit {
        players: Vec<Player>,
    },
    GameState {
        players: Vec<Player>,
        host_value: String,
        client_value: String,
    },
    GoalScored,
    GameEnded,
}


#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PlayerInput {
    pub value: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Player {
    pub number: usize,
    pub current_input: PlayerInput,
}

impl Player {
    pub fn new(number: usize) -> Self {

        Self { 
            number, 
            current_input: PlayerInput::default()
        }
    }

    pub fn set_input(&mut self, input: PlayerInput) {
        self.current_input = input;
    }

    pub fn get_input(&self) -> PlayerInput {
        self.current_input.clone()
    }
}