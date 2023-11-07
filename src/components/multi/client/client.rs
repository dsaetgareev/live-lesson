use web_sys::MouseEvent;
use yew::{html, Html, Callback, function_component, use_effect, use_state};
use yewdux::prelude::use_store;

use crate::components::common::battons::{VideoButton, AudioButton};
use crate::components::common::video::VideoBox;
use crate::components::multi::client::client_area::ClientArea;
use crate::components::multi::client::host_area::HostArea;
use crate::constants::VIDEO_ELEMENT_ID;
use crate::stores::client_store::{ClientMsg, ClientStore};
use crate::stores::media_store::{ClientMediaMsg, MediaStore};
use crate::utils::dom::get_vis_class;
use crate::media_devices::device_selector::DeviceSelector;

#[function_component(Devices)]
pub fn devices() -> Html {
    let (_state, dispatch) = use_store::<MediaStore>();

    let mic_callback: Callback<String> = {
        let dispatch = dispatch.clone();
        Callback::from(move |audio| {
            dispatch.apply(ClientMediaMsg::AudioDeviceChanged(audio))
        })
    };
    let cam_callback = {
        let dispatch = dispatch.clone();
        Callback::from(move |video| {
            dispatch.apply(ClientMediaMsg::VideoDeviceChanged(video));
        })
    };
    html! {
        <>
            <DeviceSelector on_microphone_select={mic_callback} on_camera_select={cam_callback}/>
        </>
    }
}

#[function_component(ClientVideo)]
pub fn client_video() -> Html {

    let (state, dispatch) = use_store::<MediaStore>();
    let video_enabled = use_state(|| !state.get_camera().get_enabled());
    let audio_enabled = use_state(|| !state.get_microphone().get_enabled());

    let on_video_btn = {
        let dispatch = dispatch.clone();
        let state = state.clone();
        let video_enabled = video_enabled.clone();
        Callback::from(move |_event: MouseEvent| {
            let on_video = state.get_camera().get_enabled();
            video_enabled.set(on_video);
            dispatch.apply(ClientMediaMsg::SwitchVedeo(!on_video));
        })
    };

    let on_audio_btn = {
        let dispatch = dispatch.clone();
        let state = state.clone();
        let audio_enabled = audio_enabled.clone();
        Callback::from(move |_event: MouseEvent| {
            let on_audio = state.get_microphone().get_enabled();
            audio_enabled.set(on_audio);
            dispatch.apply(ClientMediaMsg::SwitchMic(!on_audio));
        })
    };
    
    html! {
        <>
            <div class="btn-container">
                <VideoButton key={&*video_enabled.to_string()} on_btn={ on_video_btn } enabled={ *video_enabled }/>
                <AudioButton key={&*audio_enabled.to_string()} on_btn={ on_audio_btn } enabled={ *audio_enabled }/>
            </div>
            <VideoBox 
                video_id={ VIDEO_ELEMENT_ID }
                video_class={ "client_canvas vis".to_string() }
                placeholder_id={ "video-logo".to_string() }
                placeholder_class={ "unvis".to_string() }
            />
        </>
    }
}

#[function_component(ItemContent)]
pub fn item_content() -> Html {   
    let (state, _dispatch) = use_store::<MediaStore>();

    let is_visible = get_vis_class(state.is_communication());


    html! {
        <div class="content-item">
                               
            <div id="video-container" class=" vis">
                <ClientVideo />                
                <div id="video-box" class={ is_visible }>
                </div>
            </div>
            
        </div>
    }
}

#[function_component(Client)]
pub fn client() -> Html {
    let (state, dispatch) = use_store::<ClientStore>();
    use_effect({
        let dispatch = dispatch.clone();
        move || {
            match state.get_client_manager() {
                Some(_) => {
                    log::error!("clent manager");
                }
                None => {
                    log::error!("none clent manager");
                }
            }
            dispatch.apply(ClientMsg::InitClientManager);
        }
    });

    html! {
        <div id="container" class="container">
            <div class="client-container">
                <ItemContent />
                <div class="content-item">
                    <ClientArea />
                </div>
                <div class="content-item">
                    <HostArea />
                </div>
                <div class="content-item">                                             
                    <video id="render" autoplay=true class="client_canvas vis"></video>
                </div>
            </div>
            <Devices />
            <div id="shcreen_container" class="consumer unvis">
            </div>
        </div>
    }
}