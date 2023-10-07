use serde::{Serialize, Deserialize};

use crate::models::{packet::{VideoPacket, AudioPacket}, commons::AreaKind};

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
        area_kind: AreaKind,
        is_communication: bool,
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
        packet: AudioPacket
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
    },
    OnCummunication {
        message: bool
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
    Video {
        packet: VideoPacket
    }
}