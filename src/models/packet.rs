use std::sync::Arc;

use js_sys::Uint8Array;
use serde::{Serialize, Deserialize};
use web_sys::{EncodedAudioChunkInit, EncodedAudioChunk, EncodedVideoChunk, EncodedVideoChunkInit};

use crate::wrappers::{EncodedVideoChunkTypeWrapper, EncodedAudioChunkTypeWrapper};
#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct VideoPacket {
    pub data: Vec<u8>,
    pub chunk_type: String,
    pub timestamp: f64,
    pub duration: f64,
    pub sequence_number: u64,
}

impl VideoPacket {
    pub fn new(
        chunk: web_sys::EncodedVideoChunk,
        sequence_number: u64,
    ) -> Self {
        let duration = chunk.duration().expect("no duration video chunk");
        let mut buffer: [u8; 1000000] = [0; 1000000];
        let byte_length = chunk.byte_length() as usize;
        chunk.copy_to_with_u8_array(&mut buffer);
        let data = buffer[0..byte_length].to_vec();
        let chunk_type = EncodedVideoChunkTypeWrapper(chunk.type_()).to_string();
        let timestamp = chunk.timestamp();

        Self {
            data,
            chunk_type,
            timestamp,
            duration,
            sequence_number,
        }
    }

    pub fn get_encoded_video_chunk(packet: Arc<VideoPacket>) -> EncodedVideoChunk {
        let video_data = VideoPacket::get_video_data(packet);
        VideoPacket::get_encoded_video_chunk_from_data(Arc::new(video_data))
    }

    pub fn get_encoded_video_chunk_from_data(video_data: Arc<VideoPacket>) -> EncodedVideoChunk {
        let data = Uint8Array::from(video_data.data.as_ref());
        let chunk_type = EncodedVideoChunkTypeWrapper::from(video_data.chunk_type.as_str()).0;
        let mut encoded_chunk_init = EncodedVideoChunkInit::new(&data, video_data.timestamp, chunk_type);
        encoded_chunk_init.duration(video_data.duration);
        let encoded_video_chunk = EncodedVideoChunk::new(
            &encoded_chunk_init
        ).unwrap();
        encoded_video_chunk
    }

    pub fn get_video_data(packet: Arc<VideoPacket>) -> VideoPacket{
        let chunk_type = EncodedVideoChunkTypeWrapper::from(packet.chunk_type.as_str()).0;
        let video_data = Uint8Array::new_with_length(packet.data.len().try_into().unwrap());
        video_data.copy_from(&packet.data);
        let video_chunk = EncodedVideoChunkInit::new(&video_data, packet.timestamp, chunk_type);
        let chunk = EncodedVideoChunk::new(&video_chunk).unwrap();
        
        let mut video_vector = vec![0u8; chunk.byte_length() as usize];
        let video_message = video_vector.as_mut();
        chunk.copy_to_with_u8_array(video_message);
        let data = VideoPacket {
            data: video_message.to_vec(),
            chunk_type: packet.chunk_type.clone(),
            timestamp: packet.timestamp,
            duration: packet.duration,
            sequence_number: packet.sequence_number
        };
        data
    }
}

#[derive(PartialEq, Serialize, Deserialize, Clone)]
pub struct AudioPacket {
    pub data: Vec<u8>,
    pub chunk_type: String,
    pub timestamp: f64,
    pub duration: f64,
}

impl AudioPacket {
    pub fn new(chunk: web_sys::EncodedAudioChunk) -> Self {
        let duration = chunk.duration().unwrap();
        let mut buffer: [u8; 100000] = [0; 100000];
        let byte_length = chunk.byte_length() as usize;
        chunk.copy_to_with_u8_array(&mut buffer);
        let data = buffer[0..byte_length as usize].to_vec();
        let chunk_type = EncodedAudioChunkTypeWrapper(chunk.type_()).to_string();
        let timestamp = chunk.timestamp();

        Self {
            data,
            chunk_type,
            timestamp,
            duration 
        }
    }

    pub fn get_encoded_audio_chunk(packet: AudioPacket) -> EncodedAudioChunk {
        let chunk_type = EncodedAudioChunkTypeWrapper::from(packet.chunk_type).0;
        let audio_data = &packet.data;
        let audio_data_js: js_sys::Uint8Array =
            js_sys::Uint8Array::new_with_length(audio_data.len() as u32);
        audio_data_js.copy_from(audio_data.as_slice());
        let chunk_type = EncodedAudioChunkTypeWrapper(chunk_type);
        let mut audio_chunk_init =
            EncodedAudioChunkInit::new(&audio_data_js.into(), packet.timestamp, chunk_type.0);
        audio_chunk_init.duration(packet.duration);
        let encoded_audio_chunk = EncodedAudioChunk::new(&audio_chunk_init).unwrap();
        encoded_audio_chunk
    }
}