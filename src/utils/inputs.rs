use serde::{Serialize, Deserialize};

use crate::models::{packet::VideoPacket, commons::AreaKind};

#[derive(Serialize, Deserialize)]
pub enum PaintAction {
    Down,
    Move,
    Up
}

#[derive(Serialize, Deserialize)]
pub enum Message {
    Init {
        editor_content: String,
        text_area_content: String,
        area_kind: AreaKind
    },
    HostToHost {
        message: String,
        area_kind: AreaKind,
    },
    HostToClient {
        message: String,
        area_kind: AreaKind,
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
    HostSwitchAudio,
    HostSwitchVideo,
    HostSwitchArea {
        message: AreaKind
    },
    OpenPaint,
    HostPaint {
        offset_x: f64,
        offset_y: f64,
        action: PaintAction,
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AudioPacket {
    pub message: Vec<u8>,
    pub chunk_type: String,
    pub timestamp: f64,
    pub duration: f64,
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
        packet: AudioPacket
    },
    ClientSwitchVideo {
        message: bool
    },
    ClientToClient {
        message: String,
        area_kind: AreaKind,
    },
    ClientSwitchArea {
        message: AreaKind,
    }
}

#[derive(Serialize, Deserialize)]
pub enum ManyMassage {
    Audio {
        packet: AudioPacket
    },
}