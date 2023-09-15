use std::{collections::BTreeMap, sync::Arc, cmp::Ordering};

use js_sys::Uint8Array;
use web_sys::{ VideoDecoder, VideoDecoderConfig, EncodedVideoChunk, EncodedVideoChunkInit, EncodedVideoChunkType};

use crate::wrappers::EncodedVideoChunkTypeWrapper;

use super::packet::VideoPacket;

const MAX_BUFFER_SIZE: usize = 10;

#[derive(Clone)]
pub struct Video {
    pub cache: BTreeMap<u64, Arc<VideoPacket>>,
    pub video_decoder: VideoDecoder,
    pub video_config: VideoDecoderConfig,
    pub on_video: bool,
    pub video_start: bool,
    pub check_key: bool,
    pub render_id: String,
    pub sequence: Option<u64>,
}

impl Video {
    pub fn new(
        video_decoder: VideoDecoder,
        video_config: VideoDecoderConfig,
        render_id: String,
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
        }
    }

    pub fn decode_break(&mut self, packet: Arc<VideoPacket>) -> Result<(), anyhow::Error> {
        let new_sequence_number = packet.sequence_number;
        if packet.chunk_type == "key" {
            self.decode_packet(packet);
            self.sequence = Some(new_sequence_number);
            // self.prune_older_frames_from_buffer(new_sequence_number);
        } else if let Some(sequence) =self.sequence {
            let is_future_frame = new_sequence_number > sequence;
            let is_next_frame = new_sequence_number == sequence + 1;
            if is_next_frame {
                self.decode_packet(packet);
                self.sequence = Some(new_sequence_number);
                // self.prune_older_frames_from_buffer(sequence);
            } else {
                if is_future_frame {
                    // self.cache.insert(new_sequence_number, packet);
                }
            }
        }
        Ok(())
    }

    pub fn decode(&mut self, packet: Arc<VideoPacket>) -> Result<(), anyhow::Error> {
        let new_sequence_number = packet.sequence_number;
        let frame_type = EncodedVideoChunkTypeWrapper::from(packet.chunk_type.as_str()).0;
        let cache_size = self.cache.len();
        // If we get a keyframe, play it immediately, then prune all packets before it
        if frame_type == EncodedVideoChunkType::Key {
            self.decode_packet(packet);
            self.sequence = Some(new_sequence_number);
            self.prune_older_frames_from_buffer(new_sequence_number);
        } else if let Some(sequence) = self.sequence {
            let is_future_frame = new_sequence_number > sequence;
            let is_future_i_frame = is_future_frame && frame_type == EncodedVideoChunkType::Key;
            let is_next_frame = new_sequence_number == sequence + 1;
            let next_frame_already_cached = self.cache.get(&(sequence + 1)).is_some();
            if is_future_i_frame || is_next_frame {
                self.decode_packet(packet);
                self.sequence = Some(new_sequence_number);
                self.play_queued_follow_up_frames();
                self.prune_older_frames_from_buffer(sequence);
            } else {
                if next_frame_already_cached {
                    self.play_queued_follow_up_frames();
                    self.prune_older_frames_from_buffer(sequence);
                }
                if is_future_frame {
                    self.cache.insert(new_sequence_number, packet);
                    if cache_size + 1 > MAX_BUFFER_SIZE {
                        self.fast_forward_frames_and_then_prune_buffer();
                    }
                }
            }
        }
        Ok(())
    }

    pub fn decode_packet(&self, packet: Arc<VideoPacket>) {
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
            web_sys::CodecState::Unconfigured => {
                log::info!("video decoder unconfigured");
            },
            web_sys::CodecState::Configured => {
                let _ = self.video_decoder.decode(&encoded_video_chunk);
            },
            web_sys::CodecState::Closed => {
                log::info!("video decoder closed");
                // decoders.as_ref().borrow_mut().insert(
                //     user_id, 
                //     Rc::new(RefCell::new(create_video_decoder_frame(video.render_id.clone()))));
                // video.check_key = true;
            },
            _ => {},
        }
    }

    fn fast_forward_frames_and_then_prune_buffer(&mut self) {
        let mut should_skip = false;
        let sorted_frames = self.cache.keys().cloned().collect::<Vec<_>>();
        let mut to_remove = Vec::new(); // We will store the keys that we want to remove here
        for (index, sequence) in sorted_frames.iter().enumerate() {
            let image = self.cache.get(sequence).unwrap();
            let frame_type = EncodedVideoChunkTypeWrapper::from(image.chunk_type.as_str()).0;
            let next_sequence = if (index == 0 || *sequence == sorted_frames[index - 1] + 1)
                || (self.sequence.is_some()
                    && *sequence > self.sequence.unwrap()
                    && frame_type == EncodedVideoChunkType::Key)
            {
                Some(*sequence)
            } else {
                should_skip = true;
                None
            };
            if let Some(next_sequence) = next_sequence {
                if !should_skip {
                    let next_image = self.cache.get(&next_sequence).unwrap();
                    self.decode_packet(next_image.clone());
                    self.sequence = Some(next_sequence);
                    to_remove.push(next_sequence); // Instead of removing here, we add it to the remove list
                }
            } else if let Some(self_sequence) = self.sequence {
                if *sequence < self_sequence {
                    to_remove.push(*sequence); // Again, add to the remove list instead of removing directly
                }
            }
        }
        // After the iteration, we can now remove the items from the cache
        for sequence in to_remove {
            self.cache.remove(&sequence);
        }
    }

    fn prune_older_frames_from_buffer(&mut self, sequence_number: u64) {
        self.cache
            .retain(|sequence, _| *sequence >= sequence_number)
    }

    fn play_queued_follow_up_frames(&mut self) {
        let sorted_frames = self.cache.keys().collect::<Vec<_>>();
        if self.sequence.is_none() || sorted_frames.is_empty() {
            return;
        }
        for current_sequence in sorted_frames {
            let next_sequence = self.sequence.unwrap() + 1;
            match current_sequence.cmp(&next_sequence) {
                Ordering::Less => continue,
                Ordering::Equal => {
                    if let Some(next_image) = self.cache.get(current_sequence) {
                        self.decode_packet(next_image.clone());
                        self.sequence = Some(next_sequence);
                    }
                }
                Ordering::Greater => break,
            }
        }
    }
}