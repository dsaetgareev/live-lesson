use std::{rc::Rc, cell::RefCell};

use js_sys::{JsString, Reflect, Array};
use log::error;
use wasm_bindgen::{JsValue, prelude::Closure, JsCast};
use wasm_bindgen_futures::JsFuture;
use web_sys::{VideoDecoder, HtmlImageElement, HtmlCanvasElement, CanvasRenderingContext2d, VideoFrame, VideoDecoderInit, VideoDecoderConfig, AudioDecoder, MediaStreamTrackGenerator, MediaStreamTrackGeneratorInit, AudioData, AudioDecoderInit, AudioDecoderConfig, HtmlVideoElement, MediaStream};

use crate::{constants::{VIDEO_CODEC, AUDIO_CHANNELS, AUDIO_CODEC, AUDIO_SAMPLE_RATE}, models::{video::Video, audio::Audio}};

use super::{dom::{get_window, get_document, get_element}, config::configure_audio_context};

#[derive(Clone)]
pub enum VideoElementKind {
    ClentBox,
    ReadyId,
}

pub fn create_video_decoder(render_id: String) -> Video {
    let error_video = Closure::wrap(Box::new(move |e: JsValue| {
        error!("{:?}", e);
    }) as Box<dyn FnMut(JsValue)>);

    let ren_id  = render_id.clone();
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
    let video_config = VideoDecoderConfig::new(&VIDEO_CODEC);
    local_video_decoder.configure(&video_config);
    Video::new(local_video_decoder, video_config, ren_id, VideoElementKind::ReadyId)
}


pub fn create_video_decoder_frame(render_id: String) -> Video {
    let r_id = render_id.clone();
    let error_video = Closure::wrap(Box::new(move |e: JsValue| {
        error!("errorrrrrr {}", r_id.clone());
        error!("{:?}", e);
    }) as Box<dyn FnMut(JsValue)>);
    let ren_id = render_id.clone();
    let frame_count = Rc::new(RefCell::new(0));
    let output = Closure::wrap(Box::new(move |original_chunk: JsValue| {
        *frame_count.borrow_mut() += 1;
        let chunk = Box::new(original_chunk);
        let video_chunk = chunk.clone().unchecked_into::<HtmlVideoElement>();
        if *frame_count.borrow() % 3 == 0 {
            
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

            let _ = ctx.draw_image_with_html_video_element(
                &video_chunk, 
                0.0 as f64,
                0.0 as f64
            );
        }
        

        video_chunk.unchecked_into::<VideoFrame>().close();
    }) as Box<dyn FnMut(JsValue)>);

    let local_video_decoder = VideoDecoder::new(
        &VideoDecoderInit::new(error_video.as_ref().unchecked_ref(), output.as_ref().unchecked_ref())
    ).unwrap();
    error_video.forget();
    output.forget();
    let video_config = VideoDecoderConfig::new(&VIDEO_CODEC);
    local_video_decoder.configure(&video_config);
    Video::new(local_video_decoder, video_config, ren_id, VideoElementKind::ReadyId)
}

pub fn create_video_decoder_video(video_elem_id: String, el_kind: VideoElementKind) -> Video {
    
    let r_id = video_elem_id.clone();
    let err_id = video_elem_id.clone();
    let error_video = Closure::wrap(Box::new(move |e: JsValue| {
        error!("{:?}", e);
        error!("error from id: {}", err_id);
    }) as Box<dyn FnMut(JsValue)>);

    let video_stream_generator =
        MediaStreamTrackGenerator::new(&MediaStreamTrackGeneratorInit::new("video")).unwrap();
    let js_tracks = Array::new();
    js_tracks.push(&video_stream_generator);
    let media_stream = MediaStream::new_with_tracks(&js_tracks).unwrap();
    let video_element = create_video_element(video_elem_id, el_kind.clone());
    let output = Closure::wrap(Box::new(move |original_chunk: JsValue| {
        let chunk = Box::new(original_chunk);
        let video_chunk = chunk.clone().unchecked_into::<HtmlVideoElement>();
              
        // &video_element.set_width(width as u32);
        // &video_element.set_height(height as u32);
        let writable = video_stream_generator.writable();
        if writable.locked() {
            return;
        }
        if let Err(e) = writable.get_writer().map(|writer| {
            wasm_bindgen_futures::spawn_local(async move {
                if let Err(e) = JsFuture::from(writer.ready()).await {
                    error!("write chunk error {:?}", e);
                }
                if let Err(e) = JsFuture::from(writer.write_with_chunk(&video_chunk)).await {
                    error!("write chunk error {:?}", e);
                };
                video_chunk.unchecked_into::<VideoFrame>().close();
                writer.release_lock();
            });
        }) {
            error!("error {:?}", e);
        }
       
        // media_stream.add_track(&video_stream_generator);

        
    }) as Box<dyn FnMut(JsValue)>);

    
    video_element.set_src_object(Some(&media_stream));

    let local_video_decoder = VideoDecoder::new(
        &VideoDecoderInit::new(error_video.as_ref().unchecked_ref(), output.as_ref().unchecked_ref())
    ).unwrap();
    error_video.forget();
    output.forget();
    let video_config = VideoDecoderConfig::new(&VIDEO_CODEC); 
    local_video_decoder.configure(&video_config);
    Video::new(local_video_decoder, video_config, r_id, el_kind)
}

fn create_video_element(video_elem_id: String, el_kind: VideoElementKind) -> HtmlVideoElement {
    match el_kind {
        VideoElementKind::ClentBox => {
            let video_element = get_document()
                .create_element("video")
                .expect("cannot create video element")
                .dyn_into::<web_sys::HtmlVideoElement>()
                .expect("cannot cast video element");

            video_element.set_id(&video_elem_id);
            video_element.set_class_name("item-canvas");
            video_element.set_autoplay(true);
    
            let div = get_element("video-box").unwrap();
            let _ = div.append_child(&video_element);
            video_element
        },
        VideoElementKind::ReadyId => {
            let video_element = get_window().unwrap()
                .document()
                .unwrap()
                .get_element_by_id(&video_elem_id)
                .unwrap()
                .unchecked_into::<HtmlVideoElement>();
            video_element
        },
    }
}

pub fn create_audio_decoder() -> Audio {
    let error = Closure::wrap(Box::new(move |e: JsValue| {
        error!("{:?}", e);
    }) as Box<dyn FnMut(JsValue)>);
    let audio_stream_generator =
        MediaStreamTrackGenerator::new(&MediaStreamTrackGeneratorInit::new("audio")).unwrap();
    // The audio context is used to reproduce audio.
    let (audio_context, gain_node) = configure_audio_context(&audio_stream_generator).unwrap();
   
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
    error.forget();
    output.forget();
    Audio::new(audio_context, gain_node, decoder)
}