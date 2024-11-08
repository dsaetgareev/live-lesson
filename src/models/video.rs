use std::{collections::BTreeMap, sync::Arc};
use web_sys::{ VideoDecoder, VideoDecoderConfig, CodecState, HtmlVideoElement};
use crate::utils::{device::{ create_video_decoder_video, VideoElementKind, create_video_decoder_video_screen}, dom::remove_element};
use super::packet::VideoPacket;

#[derive(Clone, PartialEq)]
pub struct Video {
    pub cache: BTreeMap<u64, Arc<VideoPacket>>,
    pub video_decoder: VideoDecoder,
    pub video_config: VideoDecoderConfig,
    pub on_video: bool,
    pub video_start: bool,
    pub check_key: bool,
    pub render_id: String,
    pub sequence: Option<u64>,
    pub element_kind: VideoElementKind,
    pub require_key: bool,
    pub video_element: HtmlVideoElement,
    pub is_screen: bool,
}

impl Video {
    pub fn new(
        video_decoder: VideoDecoder,
        video_config: VideoDecoderConfig,
        render_id: String,
        element_kind: VideoElementKind,
        video_element: HtmlVideoElement,
        is_screen: bool,
    ) -> Self {
        Self {
            cache: BTreeMap::new(),
            video_decoder,
            video_config,
            on_video: true,
            video_start: true,
            check_key: true,
            render_id,
            sequence: None,
            element_kind,
            require_key: false,
            video_element,
            is_screen,
        }
    }

    pub fn decode_break(&mut self, packet: Arc<VideoPacket>) -> Result<(), anyhow::Error> {
        let new_sequence_number = packet.sequence_number;
        if packet.chunk_type == "key" {
            self.require_key = false;
            self.decode_packet(packet);
            self.sequence = Some(new_sequence_number);
        } else if let Some(sequence) =self.sequence {
            if self.require_key {
                return Ok(());
            }
            let is_next_frame = new_sequence_number == sequence + 1;
            if is_next_frame {
                self.decode_packet(packet);
                self.sequence = Some(new_sequence_number);
            }
        }
        Ok(())
    }

    pub fn decode_packet(&mut self, packet: Arc<VideoPacket>) {
        let encoded_video_chunk = VideoPacket::get_encoded_video_chunk(packet);
        match self.video_decoder.state() {
            CodecState::Unconfigured => {
                log::info!("video decoder unconfigured");
            },
            CodecState::Configured => {
                let _ = self.video_decoder.decode(&encoded_video_chunk);
            },
            CodecState::Closed => {
                log::error!("video decoder closed");
                self.require_key = true;
                
                if self.is_screen {
                    self.video_decoder = create_video_decoder_video_screen(self.render_id.clone(), self.element_kind.clone())
                    .video_decoder;
                } else {
                    remove_element(self.render_id.clone());
                    self.video_decoder = create_video_decoder_video(self.render_id.clone(), self.element_kind.clone())
                    .video_decoder;
                }
                
            },
            _ => {},
        }
    }

    pub fn decode_break_data(&mut self, packet: Arc<VideoPacket>) -> Result<(), anyhow::Error> {
        let new_sequence_number = packet.sequence_number;
        if packet.chunk_type == "key" {
            self.require_key = false;
            self.decode_packet_data(packet);
            self.sequence = Some(new_sequence_number);
        } else if let Some(sequence) = self.sequence {
            if !self.require_key && new_sequence_number == sequence + 1 {
                self.decode_packet_data(packet);
                self.sequence = Some(new_sequence_number);
            }
        }
        Ok(())
    }

    pub fn decode_packet_data(&mut self, packet: Arc<VideoPacket>) {
        let encoded_video_chunk = VideoPacket::get_encoded_video_chunk_from_data(packet);
        match self.video_decoder.state() {
            CodecState::Unconfigured => {
                log::info!("video decoder unconfigured");
            },
            CodecState::Configured => {
                let _ = self.video_decoder.decode(&encoded_video_chunk);
            },
            CodecState::Closed => {
                log::error!("video decoder closed");
                self.require_key = true;
                
                if self.is_screen {
                    self.video_decoder = create_video_decoder_video_screen(self.render_id.clone(), self.element_kind.clone())
                    .video_decoder;
                } else {
                    remove_element(self.render_id.clone());
                    self.video_decoder = create_video_decoder_video(self.render_id.clone(), self.element_kind.clone())
                    .video_decoder;
                }
                
            },
            _ => {},
        }
    }

}