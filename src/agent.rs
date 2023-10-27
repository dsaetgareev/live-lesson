use serde::{Serialize, Deserialize};
use yew_agent::{WorkerLink, Public};
use crate::models::packet::AudioPacket;

/// Used by gloo-worker, specify the worker.js file path genereated.
static WORKER_PATH: &'static str = "worker.js";

pub struct AudioWorker {
    /// link used to send messages to main thread
    link: WorkerLink<Self>,
}


#[derive(Serialize, Deserialize)]
pub struct AudioWorkerInput {
    pub file: String,
    pub packet: AudioPacket 
}

#[derive(Serialize, Deserialize)]
pub struct AudioWorkerOutput {
    pub url: String,
}

impl yew_agent::Worker for AudioWorker {
    type Reach = Public<Self>;

    type Message = ();

    type Input = AudioWorkerInput;

    type Output = AudioWorkerOutput;

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
        let url = format!("hello output id: {:?}, msg: {}", id, msg.file);
        // self.audio.decode(m)
        let output = Self::Output { url };
        self.link.respond(id, output)
    }

     fn name_of_resource() -> &'static str {
        &WORKER_PATH
    }
}