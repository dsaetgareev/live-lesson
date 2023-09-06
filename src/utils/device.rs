use js_sys::{JsString, Reflect};
use log::error;
use wasm_bindgen::{JsValue, prelude::Closure, JsCast};
use wasm_bindgen_futures::JsFuture;
use web_sys::{VideoDecoder, HtmlImageElement, HtmlCanvasElement, CanvasRenderingContext2d, VideoFrame, VideoDecoderInit, VideoDecoderConfig, AudioDecoder, MediaStreamTrackGenerator, MediaStreamTrackGeneratorInit, AudioData, AudioDecoderInit, AudioDecoderConfig};

use crate::constants::{VIDEO_CODEC, AUDIO_CHANNELS, AUDIO_CODEC, AUDIO_SAMPLE_RATE};

use super::{dom::get_window, config::configure_audio_context};

pub fn create_video_decoder(render_id: String) -> VideoDecoder {
    let error_video = Closure::wrap(Box::new(move |e: JsValue| {
        error!("{:?}", e);
    }) as Box<dyn FnMut(JsValue)>);

    let output = Closure::wrap(Box::new(move |original_chunk: JsValue| {
        let chunk = Box::new(original_chunk);
        let video_chunk = chunk.clone().unchecked_into::<HtmlImageElement>();
        let width = Reflect::get(&chunk.clone(), &JsString::from("codedWidth"))
            .unwrap()
            .as_f64()
            .unwrap();
        let height = Reflect::get(&chunk.clone(), &JsString::from("codedHeight"))
            .unwrap()
            .as_f64()
            .unwrap();

        let render_canvas = get_window()
            .unwrap()
            .document()
            .unwrap()
            .get_element_by_id(&render_id)
            .unwrap()
            .unchecked_into::<HtmlCanvasElement>();

        render_canvas.set_width(width as u32);
        render_canvas.set_height(height as u32);

        let ctx = render_canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .unchecked_into::<CanvasRenderingContext2d>();

        let _ = ctx.draw_image_with_html_image_element(
            &video_chunk, 
            0.0 as f64,
            0.0 as f64
        );

        video_chunk.unchecked_into::<VideoFrame>().close();
    }) as Box<dyn FnMut(JsValue)>);

    let local_video_decoder = VideoDecoder::new(
        &VideoDecoderInit::new(error_video.as_ref().unchecked_ref(), output.as_ref().unchecked_ref())
    ).unwrap();
    error_video.forget();
    output.forget();
    local_video_decoder.configure(&VideoDecoderConfig::new(&VIDEO_CODEC));
    local_video_decoder
}

pub fn create_audio_decoder() -> AudioDecoder {
    let error = Closure::wrap(Box::new(move |e: JsValue| {
        error!("{:?}", e);
    }) as Box<dyn FnMut(JsValue)>);
    let audio_stream_generator =
        MediaStreamTrackGenerator::new(&MediaStreamTrackGeneratorInit::new("audio")).unwrap();
    // The audio context is used to reproduce audio.
    let _audio_context = configure_audio_context(&audio_stream_generator).unwrap();

    let output = Closure::wrap(Box::new(move |audio_data: AudioData| {
        let writable = audio_stream_generator.writable();
        if writable.locked() {
            return;
        }
        if let Err(e) = writable.get_writer().map(|writer| {
            wasm_bindgen_futures::spawn_local(async move {
                if let Err(e) = JsFuture::from(writer.ready()).await {
                    error!("write chunk error {:?}", e);
                }
                if let Err(e) = JsFuture::from(writer.write_with_chunk(&audio_data)).await {
                    error!("write chunk error {:?}", e);
                };
                writer.release_lock();
            });
        }) {
            error!("error {:?}", e);
        }
    }) as Box<dyn FnMut(AudioData)>);
    let decoder = AudioDecoder::new(&AudioDecoderInit::new(
        error.as_ref().unchecked_ref(),
        output.as_ref().unchecked_ref(),
    ))
    .unwrap();
    decoder.configure(&AudioDecoderConfig::new(
        AUDIO_CODEC,
        AUDIO_CHANNELS,
        AUDIO_SAMPLE_RATE,
    ));
    decoder
}