use web_sys::{AudioContext, GainNode, AudioDecoder};


#[derive(Debug, Clone, PartialEq, Eq)]
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
}