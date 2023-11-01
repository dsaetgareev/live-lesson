
use yew::prelude::*;
use yewdux::prelude::use_store;

use crate::components::common::battons::VideoButton;
use crate::components::common::video::VideoBox;
use crate::components::multi::host::client_area::ClientArea;
use crate::components::multi::host::client_items::ClientItems;
use crate::components::multi::host::host_area::HostArea;
use crate::constants::VIDEO_ELEMENT_ID;
use crate::media_devices::device_selector::DeviceSelector;
use crate::stores::media_store::{MediaStore, HostMediaMsg};

pub enum Msg {
    Init,
    Tick,
    ChooseItem(String),
    SwitchSpeakers(String),
    SwitchVideo(String),
}

#[function_component(Devices)]
pub fn devices() -> Html {
    let (_state, dispatch) = use_store::<MediaStore>();
    let mic_callback: Callback<String> = {
        let dispatch = dispatch.clone();
        Callback::from(move |audio| {
            dispatch.apply(HostMediaMsg::AudioDeviceChanged(audio))
        })
    };
    let cam_callback = {
        let dispatch = dispatch.clone();
        Callback::from(move |video| {
            dispatch.apply(HostMediaMsg::VideoDeviceChanged(video));
        })
    };
    html! {
        <>
            <DeviceSelector on_microphone_select={mic_callback} on_camera_select={cam_callback}/>
        </>
    }
}

#[function_component(ScreenShare)]
pub fn screen_share() -> Html {
    let (_state, dispatch) = use_store::<MediaStore>();
    let screen_share_cb = {
        Callback::from(move |_| {
            log::error!("in sreen btn");
            dispatch.apply(HostMediaMsg::EnableScreenShare(true));
        })
    };
    html! {
        <div>
            <button onclick={ screen_share_cb }>{"Демонстрация экрана"}</button>
        </div>
    }
}

#[function_component(HostVideo)]
pub fn host_video() -> Html {

    let (state, dispatch) = use_store::<MediaStore>();
    let on_video_btn = {
        let dispatch = dispatch.clone();
        let state = state.clone();
        Callback::from(move |_event: MouseEvent| {
            let on_video = state.get_camera().get_enabled();
            dispatch.apply(HostMediaMsg::SwitchVedeo(!on_video));
        })
    };

    html! {
        <>
            <VideoButton { on_video_btn } enabled={ state.get_camera().get_enabled() }/>
            <VideoBox 
                video_id={ VIDEO_ELEMENT_ID }
                video_class={ "client_canvas vis".to_string() }
                placeholder_id={ "video-logo".to_string() }
                placeholder_class={ "unvis".to_string() }
            />
        </>
    }
}

#[function_component(Host)]
pub fn host() -> Html {
    html! {
        <div class="container">
            <div id="client-items" class="client-items">
                <ClientItems />
            </div>
            <div class="host-content">
                <div class="content-item">
                    <ClientArea />
                </div>
                <div class="content-item">
                    <HostArea />
                </div>
            </div>
            <div class="host-video">
                <Devices />
                <ScreenShare />
                <HostVideo />
            </div>
                   
            
            
        </div>
    }
}
