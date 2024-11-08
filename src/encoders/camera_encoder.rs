use js_sys::Array;
use js_sys::Boolean;
use js_sys::JsString;
use js_sys::Reflect;
use log::error;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::HtmlVideoElement;
use web_sys::LatencyMode;
use web_sys::MediaStream;
use web_sys::MediaStreamConstraints;
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

use crate::constants::VIDEO_CODEC;
use crate::constants::VIDEO_HEIGHT;
use crate::constants::VIDEO_WIDTH;
use crate::models::packet::VideoPacket;
use crate::utils::dom::get_window;

#[derive(Clone, PartialEq)]
pub struct CameraEncoder {
    state: EncoderState,
    device: Option<MediaStream>,
}

impl CameraEncoder {
    pub fn new() -> Self {
        Self {
            state: EncoderState::new(),
            device: None,
        }
    }

    // delegates to self.state
    pub fn set_enabled(&mut self, value: bool) -> bool {
        self.state.set_enabled(value)
    }
    pub fn get_enabled(&self) -> bool {
        self.state.is_enabled()
    }
    pub fn is_first(&self) -> bool {
        self.state.is_first()
    }
    pub fn set_first(&mut self, is_first: bool) {
        self.state.set_first(is_first);
    }
    pub fn select(&mut self, device: String) -> bool {
        self.state.select(device)
    }
    pub fn stop(&mut self) {
        self.state.stop()
    }

    pub fn init(&self, video_elem_id: &str) {
        let device_id = if let Some(vid) = &self.state.selected {
            vid.to_string()
        } else {
            return;
        };
        let video_elem_id = video_elem_id.to_string();
        wasm_bindgen_futures::spawn_local(async move {
            let navigator = get_window().unwrap().navigator();
            let video_element = get_window().unwrap()
                .document()
                .unwrap()
                .get_element_by_id(&video_elem_id)
                .unwrap()
                .unchecked_into::<HtmlVideoElement>();

            let media_devices = navigator.media_devices().unwrap();
            let mut constraints = MediaStreamConstraints::new();
            let mut media_info = web_sys::MediaTrackConstraints::new();
            media_info.device_id(&device_id.into());

            constraints.video(&media_info.into());
            constraints.audio(&Boolean::from(false));

            let devices_query = media_devices
                .get_user_media_with_constraints(&constraints)
                .unwrap();
            let device = JsFuture::from(devices_query)
                .await
                .unwrap()
                .unchecked_into::<MediaStream>();
            video_element.set_src_object(Some(&device));
            video_element.set_muted(true);
        });
    }

    pub fn start(
        &mut self,
        on_frame: impl Fn(VideoPacket) + 'static,
        video_elem_id: &str,
    ) {
        self.init(video_elem_id);
        let on_frame = Box::new(on_frame);
        let video_elem_id = video_elem_id.to_string();
        let EncoderState {
            destroy,
            enabled,
            switching,
            ..
        } = self.state.clone();
        let video_output_handler = {
            let on_frame = on_frame;
            let mut sequence_number: u64 = 0;
            Box::new(move |chunk: JsValue| {
                let chunk = web_sys::EncodedVideoChunk::from(chunk);
                let packet = VideoPacket::new(chunk, sequence_number);
                on_frame(packet);
                sequence_number += 1;
            })
        };
        let device_id = if let Some(vid) = &self.state.selected {
            vid.to_string()
        } else {
            return;
        };
        wasm_bindgen_futures::spawn_local(async move {
            let navigator = get_window().unwrap().navigator();
            let video_element = get_window().unwrap()
                .document()
                .unwrap()
                .get_element_by_id(&video_elem_id)
                .unwrap()
                .unchecked_into::<HtmlVideoElement>();

            let media_devices = navigator.media_devices().unwrap();
            let mut constraints = MediaStreamConstraints::new();
            let mut media_info = web_sys::MediaTrackConstraints::new();
            media_info.device_id(&device_id.into());

            constraints.video(&media_info.into());
            constraints.audio(&Boolean::from(false));

            let devices_query = media_devices
                .get_user_media_with_constraints(&constraints)
                .unwrap();
            let device = JsFuture::from(devices_query)
                .await
                .unwrap()
                .unchecked_into::<MediaStream>();
            video_element.set_src_object(Some(&device));
            video_element.set_muted(true);           

            let video_track = Box::new(
                device
                    .get_video_tracks()
                    .find(&mut |_: JsValue, _: u32, _: Array| true)
                    .unchecked_into::<VideoTrack>(),
            );
            
            // Setup video encoder

            let video_error_handler = Closure::wrap(Box::new(move |e: JsValue| {
                error!("error_handler error {:?}", e);
            }) as Box<dyn FnMut(JsValue)>);

            let video_output_handler =
                Closure::wrap(video_output_handler as Box<dyn FnMut(JsValue)>);

            let video_encoder_init = VideoEncoderInit::new(
                video_error_handler.as_ref().unchecked_ref(),
                video_output_handler.as_ref().unchecked_ref(),
            );

            video_error_handler.forget();
            video_output_handler.forget();

            let video_encoder = Box::new(VideoEncoder::new(&video_encoder_init).unwrap());

            let video_settings = &mut video_track
                .clone()
                .unchecked_into::<MediaStreamTrack>()
                .get_settings();
            video_settings.width(VIDEO_WIDTH);
            video_settings.height(VIDEO_HEIGHT);

            let mut video_encoder_config =
                VideoEncoderConfig::new(VIDEO_CODEC, VIDEO_HEIGHT as u32, VIDEO_WIDTH as u32);

            video_encoder_config.bitrate(100_000f64);
            video_encoder_config.latency_mode(LatencyMode::Realtime);
            video_encoder.configure(&video_encoder_config);

            let video_processor =
                MediaStreamTrackProcessor::new(&MediaStreamTrackProcessorInit::new(
                    &video_track.clone().unchecked_into::<MediaStreamTrack>(),
                ))
                .unwrap();
            let video_reader = video_processor
                .readable()
                .get_reader()
                .unchecked_into::<ReadableStreamDefaultReader>();

            // Start encoding video and audio.
            let mut video_frame_counter = 0;
            let poll_video = async {
                loop {
                    if (!*enabled.borrow())
                        || *destroy.borrow()
                        || *switching.borrow()
                    {
                        video_track
                            .clone()
                            .unchecked_into::<MediaStreamTrack>()
                            .stop();
                        video_encoder.close();
                        *switching.as_ref().borrow_mut() = false;
                        return;
                    }
                    match JsFuture::from(video_reader.read()).await {
                        Ok(js_frame) => {
                            let video_frame = Reflect::get(&js_frame, &JsString::from("value"))
                                .unwrap()
                                .unchecked_into::<VideoFrame>();
                            let mut opts = VideoEncoderEncodeOptions::new();
                            video_frame_counter = (video_frame_counter + 1) % 50;
                            opts.key_frame(video_frame_counter == 0);
                            video_encoder.encode_with_options(&video_frame, &opts);
                            video_frame.close();
                        }
                        Err(e) => {
                            error!("error {:?}", e);
                        }
                    }
                }
            };
            poll_video.await;
        });
    }
}
