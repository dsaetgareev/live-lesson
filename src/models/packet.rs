use serde::{Serialize, Deserialize};
use web_sys::{EncodedAudioChunkInit, EncodedAudioChunk};

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