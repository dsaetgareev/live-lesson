use web_sys::MouseEvent;
use yew::{html, Html, Callback, function_component, use_effect};
use yewdux::prelude::use_store;

use crate::components::common::battons::VideoButton;
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

#[function_component(ItemContent)]
pub fn item_content() -> Html {

    let (state, dispatch) = use_store::<MediaStore>();

    let is_visible = get_vis_class(state.is_communication());

    let on_video_btn = {
        let dispatch = dispatch.clone();
        Callback::from(move |event: MouseEvent| {
            dispatch.apply(ClientMediaMsg::SwitchVedeo(event));
        })
    };

    html! {
        <div class="content-item">
            <VideoButton { on_video_btn } enabled={ state.get_camera().get_enabled() }/>
            <VideoBox 
                video_id={ VIDEO_ELEMENT_ID }
                video_class={ "client_canvas vis".to_string() }
                placeholder_id={ "video-logo".to_string() }
                placeholder_class={ "unvis".to_string() }
            />
            <div id="video-box" class={ is_visible }>
            </div>
        </div>
    }
}

#[function_component(Client)]
pub fn client() -> Html {
    let (_state, dispatch) = use_store::<ClientStore>();
    use_effect({
        let dispatch = dispatch.clone();
        move || {
            dispatch.apply(ClientMsg::InitClientManager);
        }
    });

    html! {
        <div id="container" class="container">
            <div class="client-container">
                <ItemContent />
                <div class=".content-item">
                    <ClientArea />
                </div>
                <div class=".content-item">
                    <HostArea />
                </div>
                <div class=".content-item">                                             
                    <video id="render" autoplay=true class="client_canvas"></video>
                </div>
            </div>
            <Devices />
            <div id="shcreen_container" class="consumer unvis">
                <video id="screen_share" autoplay=true class="screen_canvas"></video>
            </div>
        </div>
    }
}