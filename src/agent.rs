use std::sync::Arc;

use js_sys::Uint8Array;
use serde::{Serialize, Deserialize};
use yew_agent::{WorkerLink, Public};
use crate::models::packet::VideoPacket;

/// Used by gloo-worker, specify the worker.js file path genereated.
static WORKER_PATH: &'static str = "worker.js";

pub struct VideoWorker {
    /// link used to send messages to main thread
    link: WorkerLink<Self>,
}


#[derive(Serialize, Deserialize)]
pub struct VideoWorkerInput {
    pub file: String,
    pub packet: Arc<VideoPacket> 
}

#[derive(Serialize, Deserialize)]
pub struct VideoWorkerOutput {
    pub data: VideoPacket,
}

impl yew_agent::Worker for VideoWorker {
    type Reach = Public<Self>;

    type Message = ();

    type Input = VideoWorkerInput;

    type Output = VideoWorkerOutput;

    fn create(link: WorkerLink<Self>) -> Self {
        Self { 
            link,
        }
    }

    fn update(&mut self, _msg: Self::Message) {
        todo!()
    }

    fn handle_input(&mut self, msg: Self::Input, id: yew_agent::HandlerId) {
        log::info!("message from another id: {:?}, msg: {}", id, msg.file);
        let data = VideoPacket::get_video_data(msg.packet);
        let url = "kdfjdkf".to_string();
        let output = Self::Output { data };
        self.link.respond(id, output)
    }

     fn name_of_resource() -> &'static str {
        &WORKER_PATH
    }
}