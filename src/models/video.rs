use std::{collections::BTreeMap, sync::Arc};

use js_sys::Uint8Array;
use web_sys::{ VideoDecoder, VideoDecoderConfig, EncodedVideoChunk, EncodedVideoChunkInit, CodecState, HtmlVideoElement};

use crate::{wrappers::EncodedVideoChunkTypeWrapper, utils::{device::{ create_video_decoder_video, VideoElementKind, create_video_decoder_video_screen}, dom::remove_element}};

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
        let chunk_type = EncodedVideoChunkTypeWrapper::from(packet.chunk_type.as_str()).0;
        let video_data = Uint8Array::new_with_length(packet.data.len().try_into().unwrap());
        video_data.copy_from(&packet.data);
        let video_chunk = EncodedVideoChunkInit::new(&video_data, packet.timestamp, chunk_type);
        let chunk = EncodedVideoChunk::new(&video_chunk).unwrap();
        
        let mut video_vector = vec![0u8; chunk.byte_length() as usize];
        let video_message = video_vector.as_mut();
        chunk.copy_to_with_u8_array(video_message);
        let data = Uint8Array::from(video_message.as_ref());
        let mut encoded_chunk_init = EncodedVideoChunkInit::new(&data, chunk.timestamp(), chunk.type_());
        encoded_chunk_init.duration(packet.duration);
        let encoded_video_chunk = EncodedVideoChunk::new(
            &encoded_chunk_init
        ).unwrap();
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