use web_sys::{AudioContext, GainNode, AudioDecoder, VideoDecoder, VideoDecoderConfig, EncodedVideoChunk};


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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Video {
    pub video_decoder: VideoDecoder,
    pub video_config: VideoDecoderConfig,
    pub on_video: bool,
    pub video_start: bool,
    pub check_key: bool,
}

impl Video {
    pub fn new(
        video_decoder: VideoDecoder,
        video_config: VideoDecoderConfig
    ) -> Self {
        Self {
            video_decoder,
            video_config,
            on_video: true,
            video_start: true,
            check_key: false,
        }
    }

    pub fn decode(&mut self, chunk: &EncodedVideoChunk) -> Result<(), anyhow::Error> {
        self.video_decoder.decode(chunk);
        Ok(())
    }
}