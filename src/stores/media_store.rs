use std::{rc::Rc, cell::RefCell};

use gloo_timers::callback::Timeout;
use wasm_peers::UserId;
use web_sys::MouseEvent;
use yewdux::{store::{Store, Reducer}, prelude::Dispatch};

use crate::{encoders::{camera_encoder::CameraEncoder, microphone_encoder::MicrophoneEncoder, screen_encoder::ScreenEncoder}, stores::client_store::{ClientStore, ClientMsg}, utils::{inputs::{ManyMassage, ClientMessage, Message}, dom::{on_visible_el, switch_visible_el}}, models::packet::{AudioPacket, VideoPacket}, constants::VIDEO_ELEMENT_ID};

use super::host_store::{HostStore, self};



#[derive(Clone, PartialEq, Store)]
pub struct MediaStore {
    camera: Option<CameraEncoder>,
    microphone: Option<MicrophoneEncoder>,
    screen: Option<ScreenEncoder>,
    is_communication: Rc<RefCell<bool>>,
    is_screen: bool,
}

impl Default for MediaStore {
    fn default() -> Self {
        Self { 
            camera: Some(CameraEncoder::new()),
            microphone: Some(MicrophoneEncoder::new()),
            screen: Some(ScreenEncoder::new()),
            is_communication: Rc::new(RefCell::new(true)),
            is_screen: false,
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

    pub fn get_screen(&self) -> &ScreenEncoder {
        self.screen.as_ref().unwrap()
    }

    pub fn get_mut_screen(&mut self) -> &mut ScreenEncoder {
        self.screen.as_mut().unwrap()
    }

    pub fn set_communication(&mut self, is_communication: bool) {
        self.is_communication.replace(is_communication);
    }
    pub fn is_communication(&self) -> bool {
        *self.is_communication.borrow()
    }
}

pub enum HostMediaMsg {
    AudioDeviceChanged(String),
    EnableMicrophone(bool),
    VideoDeviceChanged(String),
    EnableVideo(bool),
    OnCummunication (bool),
    EnableScreenShare(bool),
    IsScreen(UserId),
    ResumeVideo,
}

impl Reducer<MediaStore> for HostMediaMsg {
    fn apply(self, mut store: Rc<MediaStore>) -> Rc<MediaStore> {
        let state = Rc::make_mut(&mut store);
        let dispatch = Dispatch::<MediaStore>::new();
        let global_dispatch = Dispatch::<HostStore>::new();
        match self {
            HostMediaMsg::AudioDeviceChanged(audio) => {
                if state.get_mut_microphone().select(audio) {
                    let timeout = Timeout::new(1000, move || {
                        dispatch.apply(HostMediaMsg::EnableMicrophone(true));
                    });
                    timeout.forget();
                }
            },
            HostMediaMsg::EnableMicrophone(should_enable) => {
                if should_enable {
                    let global_dispatch = global_dispatch.clone();
                    let on_audio = move |chunk: web_sys::EncodedAudioChunk| {
                        
                        let audio_packet = AudioPacket::new(chunk);
                        let message = Message::HostAudio { 
                            packet: audio_packet
                        };
                        global_dispatch.apply(host_store::Msg::SendMessage(message));              
                    };              
                    state.microphone.as_mut().unwrap().start(
                        on_audio
                    );
                }               
            },
            HostMediaMsg::VideoDeviceChanged(video) => {
                if state.get_mut_camera().select(video) {
                    let timeout = Timeout::new(1000, move || {
                        dispatch.apply(HostMediaMsg::EnableVideo(true));
                    });
                    timeout.forget();
                }
            },
            HostMediaMsg::EnableVideo(should_enable) => {
                if should_enable {
                    let global_dispatch = global_dispatch.clone();
                    let on_frame = move |packet: VideoPacket| {
                        let message = Message::HostVideo { 
                            message: packet
                        };
                        global_dispatch.apply(host_store::Msg::SendMessage(message));                        
                    };
                    state.camera.as_mut().unwrap().start(
                        on_frame,
                        VIDEO_ELEMENT_ID,
                    );
                }
            },
            HostMediaMsg::EnableScreenShare(should_enable) => {
                if should_enable {
                    // state.camera.as_mut().unwrap().set_enabled(false); todo

                
                    let global_dispatch_move = global_dispatch.clone();
                    state.is_screen = true;
                    let message = Message::HostIsScreenShare { message: state.is_screen };
                    global_dispatch_move.apply(host_store::Msg::SendMessage(message));
                    let on_frame = move |packet: VideoPacket| {
                        
                        let message = Message::HostScreenShare { 
                            message: packet,
                        };
                        global_dispatch_move.apply(host_store::Msg::SendMessage(message));                
                    };

                     let global_dispatch = global_dispatch.clone();
                    let on_stop_share = move || {
                        dispatch.apply(HostMediaMsg::ResumeVideo);
                        let message = Message::HostIsScreenShare { message: false };
                        global_dispatch.apply(host_store::Msg::SendMessage(message));  
                    };
                    state.get_mut_screen().start(
                        on_frame,
                        on_stop_share,
                    );
                }
            }
            HostMediaMsg::IsScreen(user_id) => {
                let message = Message::HostIsScreenShare { message: state.is_screen };
                global_dispatch.apply(host_store::Msg::SendMessageToUser(user_id, message));
            }
            HostMediaMsg::ResumeVideo => {
                state.get_mut_camera().set_enabled(true);
            }
            HostMediaMsg::OnCummunication(message) => {
                switch_visible_el(message, "video-box");
                state.set_communication(message);
            }
        }
        store
    }

}



pub enum ClientMediaMsg {
    AudioDeviceChanged(String),
    EnableMicrophone(bool),
    VideoDeviceChanged(String),
    EnableVideo(bool),
    SwitchVedeo(MouseEvent),
    OnCummunication (bool),
    SetCommunication(bool),
}

impl Reducer<MediaStore> for ClientMediaMsg {
    fn apply(self, mut store: Rc<MediaStore>) -> Rc<MediaStore> {
        let state = Rc::make_mut(&mut store);
        let dispatch = Dispatch::<MediaStore>::new();
        let global_dispatch = Dispatch::<ClientStore>::new();
        match self {
            ClientMediaMsg::AudioDeviceChanged(audio) => {
                if state.get_mut_microphone().select(audio) {
                    let timeout = Timeout::new(1000, move || {
                        dispatch.apply(ClientMediaMsg::EnableMicrophone(true));
                    });
                    timeout.forget();
                }
            },
            ClientMediaMsg::EnableMicrophone(should_enable) => {
                if should_enable {
                    let global_dispatch = global_dispatch.clone();
                    let is_communication = state.is_communication.clone();
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
            ClientMediaMsg::VideoDeviceChanged(video) => {
                if state.get_mut_camera().select(video) {
                    let timeout = Timeout::new(1000, move || {
                        dispatch.apply(ClientMediaMsg::EnableVideo(true));
                    });
                    timeout.forget();
                }
            },
            ClientMediaMsg::EnableVideo(should_enable) => {
                if should_enable {
                    let global_dispatch = global_dispatch.clone();
                    let is_communication = state.is_communication.clone();
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
            ClientMediaMsg::SwitchVedeo(_event) => {
                let on_video = state.get_camera().get_enabled();
                let on_video = state.get_mut_camera().set_enabled(!on_video);
                let is_video = !state.get_camera().get_enabled();
                on_visible_el(is_video, VIDEO_ELEMENT_ID, "video-logo");
                let message = ClientMessage::ClientSwitchVideo { message: is_video };
                global_dispatch.apply(ClientMsg::SendMessage(message));
                
                if state.get_camera().get_enabled() {
                    let dispatch = dispatch.clone();
                    let timeout = Timeout::new(1000, move || {
                        dispatch.apply(ClientMediaMsg::EnableVideo(on_video));
                    });
                    timeout.forget();
                }
            },
            ClientMediaMsg::OnCummunication(message) => {
                switch_visible_el(message, "video-box");
                state.set_communication(message);
            }
            ClientMediaMsg::SetCommunication(message) => {
                state.set_communication(message);
            }
        }
        store
    }

}

