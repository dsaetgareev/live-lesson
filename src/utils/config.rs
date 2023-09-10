use js_sys::Array;
use serde::Deserialize;
use web_sys::{AudioContext, AudioContextOptions, OscillatorType, GainNode};
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

fn get_local_audio_context() {
    let ctx = web_sys::AudioContext::new().unwrap();

        // Create our web audio objects.
        let primary = ctx.create_oscillator().unwrap();
        let fm_osc = ctx.create_oscillator().unwrap();
        let gain = ctx.create_gain().unwrap();
        let fm_gain = ctx.create_gain().unwrap();

        // Some initial settings:
        primary.set_type(OscillatorType::Sine);
        primary.frequency().set_value(100.0); // A4 note
        gain.gain().set_value(10.0); // starts muted
        fm_gain.gain().set_value(0.0); // no initial frequency modulation
        fm_osc.set_type(OscillatorType::Sine);
        fm_osc.frequency().set_value(0.0);

        // Connect the nodes up!

        // The primary oscillator is routed through the gain node, so that
        // it can control the overall output volume.
        primary.connect_with_audio_node(&gain).unwrap();

        // Then connect the gain node to the AudioContext destination (aka
        // your speakers).
        gain.connect_with_audio_node(&ctx.destination()).unwrap();

        // The FM oscillator is connected to its own gain node, so it can
        // control the amount of modulation.
        fm_osc.connect_with_audio_node(&fm_gain).unwrap();

        // Connect the FM oscillator to the frequency parameter of the main
        // oscillator, so that the FM node can modulate its frequency.
        fm_gain.connect_with_audio_param(&primary.frequency()).unwrap();

        // Start the oscillators!
        primary.start().unwrap();
        // fm_osc.start().unwrap();
}
