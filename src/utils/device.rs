
use std::{rc::Rc, cell::RefCell};

use js_sys::Array;
use log::error;
use wasm_bindgen::{JsValue, prelude::Closure, JsCast};
use wasm_bindgen_futures::JsFuture;
use web_sys::{VideoDecoder, VideoFrame, VideoDecoderInit, VideoDecoderConfig, AudioDecoder, MediaStreamTrackGenerator, MediaStreamTrackGeneratorInit, AudioData, AudioDecoderInit, AudioDecoderConfig, HtmlVideoElement, MediaStream};

use crate::{constants::{VIDEO_CODEC, AUDIO_CHANNELS, AUDIO_CODEC, AUDIO_SAMPLE_RATE}, models::{video::Video, audio::Audio}};

use super::{dom::{get_window, get_document, get_element}, config::configure_audio_context};

#[derive(Clone, PartialEq)]
pub enum VideoElementKind {
    HostBox,
    ClentBox,
    ReadyId,
    ScreenBox,
}


pub fn create_video_decoder_video_screen(video_elem_id: String, el_kind: VideoElementKind) -> Video {
    
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
    let frame_count = Rc::new(RefCell::new(0));
    let output = Closure::wrap(Box::new(move |original_chunk: JsValue| {
        *frame_count.borrow_mut() += 1;
        if *frame_count.borrow() % 6 == 0 {
            let chunk = Box::new(original_chunk);
            let video_chunk = chunk.clone().unchecked_into::<HtmlVideoElement>();
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
        } else {
            original_chunk.unchecked_into::<VideoFrame>().close();
        }    
    }) as Box<dyn FnMut(JsValue)>);

    
    video_element.set_src_object(Some(&media_stream));

    let local_video_decoder = VideoDecoder::new(
        &VideoDecoderInit::new(error_video.as_ref().unchecked_ref(), output.as_ref().unchecked_ref())
    ).unwrap();
    error_video.forget();
    output.forget();
    let video_config = VideoDecoderConfig::new(&VIDEO_CODEC); 
    local_video_decoder.configure(&video_config);
    Video::new(local_video_decoder, video_config, r_id, el_kind, video_element, true)
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
    }) as Box<dyn FnMut(JsValue)>);

    
    video_element.set_src_object(Some(&media_stream));

    let local_video_decoder = VideoDecoder::new(
        &VideoDecoderInit::new(error_video.as_ref().unchecked_ref(), output.as_ref().unchecked_ref())
    ).unwrap();
    error_video.forget();
    output.forget();
    let video_config = VideoDecoderConfig::new(&VIDEO_CODEC); 
    local_video_decoder.configure(&video_config);
    Video::new(local_video_decoder, video_config, r_id, el_kind, video_element, false)
}

fn create_video_element(video_elem_id: String, el_kind: VideoElementKind) -> HtmlVideoElement {
    match el_kind {
        VideoElementKind::HostBox => {
            let video_element = get_document()
                .create_element("video")
                .expect("cannot create video element")
                .dyn_into::<web_sys::HtmlVideoElement>()
                .expect("cannot cast video element");

            video_element.set_id(&video_elem_id);
            video_element.set_class_name("item-canvas vis");
            video_element.set_autoplay(true);
            let box_id = format!("item-box-{}", video_elem_id);
            match get_element(&box_id) {
                Ok(element) => {
                    let _ = element.append_child(&video_element);
                },
                Err(err) => {
                    log::error!("not found {}, {}", box_id, err);
                },
            };                      
            video_element
        }
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
        VideoElementKind::ScreenBox => {
            match get_element(&video_elem_id) {
                Ok(video_element) => {
                    video_element.unchecked_into::<HtmlVideoElement>()
                },
                Err(_) => {
                    let video_element = get_document()
                        .create_element("video")
                        .expect("cannot create video element")
                        .dyn_into::<web_sys::HtmlVideoElement>()
                        .expect("cannot cast video element");

                    video_element.set_id(&video_elem_id);
                    video_element.set_class_name("screen_canvas");
                    video_element.set_autoplay(true);
            
                    let div = get_element("shcreen_container").unwrap();
                    let _ = div.append_child(&video_element);
                    video_element
                },
            }
            
        }
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