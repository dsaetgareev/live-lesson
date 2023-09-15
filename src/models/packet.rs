use serde::{Serialize, Deserialize};

use crate::wrappers::EncodedVideoChunkTypeWrapper;


#[derive(Serialize, Deserialize)]
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
        let mut buffer: [u8; 100000] = [0; 100000];
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