use web_sys::{AudioContext, GainNode, AudioDecoder};
use yew::Properties;

use super::packet::AudioPacket;


#[derive(Debug, Clone, PartialEq, Eq, Properties)]
pub struct Audio {
    pub audio_context: AudioContext,
    pub gain_node: GainNode,
    pub audio_decoder: AudioDecoder,
    pub on_speakers: bool,
    pub on_video: bool,
}

impl Audio {
    pub fn new(
        audio_context: AudioContext,
        gain_node: GainNode,
        audio_decoder: AudioDecoder,
    ) -> Self {
        Self { 
            audio_context,
            gain_node,
            audio_decoder,
            on_speakers: true,
            on_video: true,
        }
    }

    pub fn decode(&self, packet: AudioPacket) {
        let encoded_audio_chunk = AudioPacket::get_encoded_audio_chunk(packet);
        let state = self.audio_decoder.state();
        log::error!("doc q s {}", self.audio_decoder.decode_queue_size());
        if self.audio_decoder.decode_queue_size() < 3 {
            match state {
                web_sys::CodecState::Unconfigured => {
                    log::info!("audio decoder unconfigured");
                },
                web_sys::CodecState::Configured => {
                    log::error!("doc q s {}", self.audio_decoder.decode_queue_size());
                    self.audio_decoder.decode(&encoded_audio_chunk);
                },
                web_sys::CodecState::Closed => {
                    log::info!("audio_decoder closed");
                },
                _ => {}
            }    
        } else {
            self.audio_decoder.reset();
        }
        
    }
}