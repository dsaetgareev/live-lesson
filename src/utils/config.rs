use js_sys::Array;
use serde::Deserialize;
use web_sys::{AudioContext, AudioContextOptions, GainNode};
use web_sys::{MediaStream, MediaStreamTrackGenerator};

use crate::constants::AUDIO_SAMPLE_RATE;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub signaling_server_url: String,
    pub stun_server_urls: String,
    pub turn_server_urls: String,
    pub turn_server_username: String,
    pub turn_server_credential: String,
}

pub fn configure_audio_context(
    audio_stream_generator: &MediaStreamTrackGenerator,
) -> anyhow::Result<(AudioContext, GainNode)> {
    // let _ = get_local_audio_context();
    let js_tracks = Array::new();
    js_tracks.push(audio_stream_generator);
    let media_stream = MediaStream::new_with_tracks(&js_tracks).unwrap();
    let mut audio_context_options = AudioContextOptions::new();
    audio_context_options.sample_rate(AUDIO_SAMPLE_RATE as f32);
    let audio_context = AudioContext::new_with_context_options(&audio_context_options).unwrap();
    let gain_node = audio_context.create_gain().unwrap();
    gain_node.set_channel_count(1);
    let source = audio_context
        .create_media_stream_source(&media_stream)
        .unwrap();
    let _ = source.connect_with_audio_node(&gain_node).unwrap();
    let _ = gain_node
        .connect_with_audio_node(&audio_context.destination())
        .unwrap();
    Ok((audio_context, gain_node))
}

