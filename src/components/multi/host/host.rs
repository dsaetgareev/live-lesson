
use yew::prelude::*;
use yewdux::prelude::use_store;

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
            dispatch.apply(HostMediaMsg::EnableScreenShare(true));
        })
    };
    html! {
        <div>
            <button onclick={ screen_share_cb }>{"Демонстрация экрана"}</button>
        </div>
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
                <video class="client_canvas" autoplay=true id={VIDEO_ELEMENT_ID}></video>
            </div>
                   
            
            
        </div>
    }
}
