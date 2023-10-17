use std::{rc::Rc, cell::RefCell};

use gloo_timers::callback::Timeout;
use web_sys::MouseEvent;
use yewdux::{store::{Store, Reducer}, prelude::Dispatch};

use crate::{encoders::{camera_encoder::CameraEncoder, microphone_encoder::MicrophoneEncoder}, stores::client_store::{ClientStore, ClientMsg}, utils::{inputs::{ManyMassage, ClientMessage}, dom::{on_visible_el, switch_visible_el}}, models::packet::{AudioPacket, VideoPacket}, constants::VIDEO_ELEMENT_ID};



#[derive(Clone, PartialEq, Store)]
pub struct MediaStore {
    camera: Option<CameraEncoder>,
    microphone: Option<MicrophoneEncoder>,
    is_communication: bool,
}

impl Default for MediaStore {
    fn default() -> Self {
        Self { 
            camera: Some(CameraEncoder::new()),
            microphone: Some(MicrophoneEncoder::new()),
            is_communication: true,
        }
    }
}

impl MediaStore {
    pub fn get_camera(&self) -> &CameraEncoder {
        self.camera.as_ref().unwrap()
    }

    pub fn get_mut_camera(&mut self) -> &mut CameraEncoder {
        self.camera.as_mut().unwrap()
    }

    pub fn get_microphone(&self) -> &MicrophoneEncoder {
        self.microphone.as_ref().unwrap()
    }

    pub fn get_mut_microphone(&mut self) -> &mut MicrophoneEncoder {
        self.microphone.as_mut().unwrap()
    }

    pub fn set_communication(&mut self, is_communication: bool) {
        self.is_communication = is_communication;
    }
    pub fn is_communication(&self) -> bool {
        self.is_communication
    }
}

pub enum MediaMsg {
    AudioDeviceChanged(String),
    EnableMicrophone(bool),
    VideoDeviceChanged(String),
    EnableVideo(bool),
    SwitchVedeo(MouseEvent),
    OnCummunication (bool),
}

impl Reducer<MediaStore> for MediaMsg {
    fn apply(self, mut store: Rc<MediaStore>) -> Rc<MediaStore> {
        let state = Rc::make_mut(&mut store);
        let dispatch = Dispatch::<MediaStore>::new();
        let global_dispatch = Dispatch::<ClientStore>::new();
        match self {
            MediaMsg::AudioDeviceChanged(audio) => {
                if state.get_mut_microphone().select(audio) {
                    let timeout = Timeout::new(1000, move || {
                        dispatch.apply(MediaMsg::EnableMicrophone(true));
                    });
                    timeout.forget();
                }
            },
            MediaMsg::EnableMicrophone(should_enable) => {
                if should_enable {
                    let global_dispatch = global_dispatch.clone();
                    let is_communication = Rc::new(RefCell::new(state.is_communication)).clone();
                    let on_audio = move |chunk: web_sys::EncodedAudioChunk| {
                        
                        let audio_packet = AudioPacket::new(chunk);
                        let message = ClientMessage::ClientAudio { 
                            packet: audio_packet.clone()
                        };
                        global_dispatch.apply(ClientMsg::SendMessage(message));
                        if *is_communication.borrow() {
                            let message = ManyMassage::Audio { 
                                packet: audio_packet
                            };
                            global_dispatch.apply(ClientMsg::SendManyMessage(message));
                        }                                
                    };
                    state.get_mut_microphone().start(
                        on_audio
                    );                
                }
            },
            MediaMsg::VideoDeviceChanged(video) => {
                if state.get_mut_camera().select(video) {
                    let timeout = Timeout::new(1000, move || {
                        dispatch.apply(MediaMsg::EnableVideo(true));
                    });
                    timeout.forget();
                }
            },
            MediaMsg::EnableVideo(should_enable) => {
                if should_enable {
                    let global_dispatch = global_dispatch.clone();
                    let is_communication = Rc::new(RefCell::new(state.is_communication)).clone();
                    let on_frame = move |packet: VideoPacket| {
                                        
                        let message = ClientMessage::ClientVideo { 
                            message: packet.clone()
                        };
                        global_dispatch.apply(ClientMsg::SendMessage(message));
                        if *is_communication.borrow() {
                            let message = ManyMassage::Video { packet };
                            global_dispatch.apply(ClientMsg::SendManyMessage(message));
                        }
                    };
                    state.get_mut_camera().start(
                        on_frame,
                        VIDEO_ELEMENT_ID,
                    );
                }              
            },
            MediaMsg::SwitchVedeo(_event) => {
                let on_video = state.get_camera().get_enabled();
                let on_video = state.get_mut_camera().set_enabled(!on_video);
                let is_video = !state.get_camera().get_enabled();
                on_visible_el(is_video, VIDEO_ELEMENT_ID, "video-logo");
                let message = ClientMessage::ClientSwitchVideo { message: is_video };
                global_dispatch.apply(ClientMsg::SendMessage(message));
                
                if state.get_camera().get_enabled() {
                    let dispatch = dispatch.clone();
                    let timeout = Timeout::new(1000, move || {
                        dispatch.apply(MediaMsg::EnableVideo(on_video));
                    });
                    timeout.forget();
                }
            },
            MediaMsg::OnCummunication(message) => {
                switch_visible_el(message, "video-box");
                state.set_communication(message);
            }
        }
        store
    }

}

