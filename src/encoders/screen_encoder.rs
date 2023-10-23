use js_sys::Array;
use js_sys::JsString;
use js_sys::Reflect;
use log::error;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::LatencyMode;
use web_sys::MediaStream;
use web_sys::MediaStreamTrack;
use web_sys::MediaStreamTrackProcessor;
use web_sys::MediaStreamTrackProcessorInit;
use web_sys::ReadableStreamDefaultReader;
use web_sys::VideoEncoder;
use web_sys::VideoEncoderConfig;
use web_sys::VideoEncoderEncodeOptions;
use web_sys::VideoEncoderInit;
use web_sys::VideoFrame;
use web_sys::VideoTrack;

use super::encoder_state::EncoderState;

use crate::constants::SCREEN_VIDEO_HEIGHT;
use crate::constants::SCREEN_VIDEO_WIDTH;
use crate::constants::VIDEO_CODEC;
use crate::models::packet::VideoPacket;
use crate::utils;

#[derive(Clone, PartialEq)]
pub struct ScreenEncoder {
    state: EncoderState,
}

impl ScreenEncoder {
    pub fn new() -> Self {
        Self {
            state: EncoderState::new(),
        }
    }

    // delegates to self.state
    pub fn set_enabled(&mut self, value: bool) -> bool {
        self.state.set_enabled(value)
    }
    pub fn stop(&mut self) {
        self.state.stop()
    }

    pub fn start(
        &mut self, 
        on_frame: impl Fn(VideoPacket) + 'static,
        on_stop_share: impl Fn() + 'static,
    ) {
        let EncoderState {
            enabled, destroy, ..
        } = self.state.clone();
        let on_frame = Box::new(on_frame);
        let screen_output_handler = {
            let on_frame = on_frame;
            let mut sequence_number: u64 = 0;
            Box::new(move |chunk: JsValue| {
                let chunk = web_sys::EncodedVideoChunk::from(chunk);
                if chunk.byte_length() < 200_000 {
                    let packet = VideoPacket::new(chunk, sequence_number);
                    on_frame(packet);
                   
                }  
                 sequence_number += 1;              
            })
        };
        wasm_bindgen_futures::spawn_local(async move {
            let navigator = utils::dom::get_window().unwrap().navigator();
            let media_devices = navigator.media_devices().unwrap();
            let screen_to_share: MediaStream =
                JsFuture::from(media_devices.get_display_media().unwrap())
                    .await
                    .unwrap()
                    .unchecked_into::<MediaStream>();

            let screen_track = Box::new(
                screen_to_share
                    .get_video_tracks()
                    .find(&mut |_: JsValue, _: u32, _: Array| true)
                    .unchecked_into::<VideoTrack>(),
            );

            let screen_error_handler = Closure::wrap(Box::new(move |e: JsValue| {
                error!("error_handler error {:?}", e);
            }) as Box<dyn FnMut(JsValue)>);

            let screen_output_handler =
                Closure::wrap(screen_output_handler as Box<dyn FnMut(JsValue)>);

            let screen_encoder_init = VideoEncoderInit::new(
                screen_error_handler.as_ref().unchecked_ref(),
                screen_output_handler.as_ref().unchecked_ref(),
            );

            screen_output_handler.forget();
            screen_error_handler.forget();

            let screen_encoder = Box::new(VideoEncoder::new(&screen_encoder_init).unwrap());
            let mut screen_encoder_config =
                VideoEncoderConfig::new(VIDEO_CODEC, SCREEN_VIDEO_HEIGHT as u32, SCREEN_VIDEO_WIDTH as u32);
            screen_encoder_config.bitrate(300_000f64);
            screen_encoder_config.latency_mode(LatencyMode::Quality);
            screen_encoder.configure(&screen_encoder_config);

            let screen_processor =
                MediaStreamTrackProcessor::new(&MediaStreamTrackProcessorInit::new(
                    &screen_track.unchecked_into::<MediaStreamTrack>(),
                ))
                .unwrap();

            let screen_reader = screen_processor
                .readable()
                .get_reader()
                .unchecked_into::<ReadableStreamDefaultReader>();

            let mut screen_frame_counter = 0;

            let poll_screen = async {
                loop {
                    if *destroy.borrow() {
                        return;
                    }
                    if !*enabled.borrow() {
                        return;
                    }
                    match JsFuture::from(screen_reader.read()).await {
                        Ok(js_frame) => {
                            let video_frame = Reflect::get(&js_frame, &JsString::from("value"))
                                .unwrap()
                                .unchecked_into::<VideoFrame>();
                            let mut opts = VideoEncoderEncodeOptions::new();
                            screen_frame_counter = (screen_frame_counter + 1) % 50;
                            opts.key_frame(screen_frame_counter == 0);
                            if video_frame.is_undefined() {
                                on_stop_share();
                                return;
                            }
                            screen_encoder.encode_with_options(&video_frame, &opts);
                            video_frame.close();
                        }
                        Err(e) => {
                            error!("rerererererererere");
                            error!("error {:?}", e);
                        }
                    }
                }
            };
            poll_screen.await;
        });
    }
}
