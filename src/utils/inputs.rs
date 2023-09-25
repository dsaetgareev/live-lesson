use serde::{Serialize, Deserialize};

use crate::models::{packet::VideoPacket, commons::AreaKind};

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
        message: VideoPacket
    },
    HostIsScreenShare {
        message: bool,
    },
    HostScreenShare {
        message: VideoPacket
    },
    HostAudio {
        message: Vec<u8>,
        chunk_type: String,
        timestamp: f64,
        duration: f64,
    },
    HostSwicthAudio,
    HostSwicthVideo,
    HostSwicthArea {
        message: AreaKind
    }
}

#[derive(Serialize, Deserialize)]
pub enum ClientMessage {
    ClientText {
        message: String,
    },

    ClientVideo {
        message: VideoPacket
    },
    ClientAudio {
        message: Vec<u8>,
        chunk_type: String,
        timestamp: f64,
        duration: f64,
    },
}