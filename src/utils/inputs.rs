use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub enum Message {
    Init {
        message: String,
    },
    HostToHost {
        message: String
    },
    HostToClient {
        message: String
    },
    HostVideo {
        message: Vec<u8>,
        chunk_type: String,
        timestamp: f64,
    },
    HostScreenShare {
        message: Vec<u8>,
        chunk_type: String,
        timestamp: f64,
    },
    HostAudio {
        message: Vec<u8>,
        chunk_type: String,
        timestamp: f64,
        duration: f64,
    }
}

#[derive(Serialize, Deserialize)]
pub enum ClientMessage {
    ClientText {
        message: String,
    },

    ClientVideo {
        message: Vec<u8>,
        chunk_type: String,
        timestamp: f64,
    },
}