use std::str::FromStr;

use wasm_peers::{SessionId, get_random_session_id};
use web_sys::MouseEvent;
use yew::{use_state, function_component, Html, html, Callback};
use yewdux::prelude::use_store;

use crate::{stores::{client_store::{ClientStore, ClientMsg}, media_store::{MediaStore, ClientMediaMsg}}, utils, components::multi::client::client::{Client, ClientVideo}, media_devices::device_selector::DeviceSelector};


#[function_component(WelcomeClient)]
pub fn welcome_host() -> Html {

    let session_id = use_state(|| {
        let query_params = utils::dom::get_query_params_multi();
        let session_id = match query_params.get("session_id") {
            Some(session_string) => {
                SessionId::new(uuid::Uuid::from_str(&session_string).unwrap().as_u128())
            }
            _ => {
                let location = utils::dom::global_window().location();
                let generated_session_id = get_random_session_id();
                query_params.append("session_id", &generated_session_id.to_string());
                // query_params.append("host", "true");
                let search: String = query_params.to_string().into();
                if let Err(error) = location.set_search(&search) {
                    log::error!("Error while setting URL: {error:?}")
                }
                generated_session_id
            }
        };
        session_id
    });

    let (_state, dispatch) = use_store::<ClientStore>();
    let to_client = use_state(|| false);

    let on_init = {
        let to_client = to_client.clone();
        let session_id = session_id.clone();
        let dispatch = dispatch.clone();
        move |_e: MouseEvent| {
            dispatch.apply(ClientMsg::Init(*session_id));
            to_client.set(true);
        }
    };
    html! {
        if *to_client {
            <Client />
            
        } else {
            <>
                <ClientVideo />
                <button onclick={ on_init }>
                    { 
                        "Подключиться к встрече"
                    }                    
                </button>
                <Devices />
                <div id="shcreen_container" class="consumer unvis">
                </div>
            </>
            
        }
    }
}


#[function_component(Devices)]
pub fn devices() -> Html {
    let (_state, dispatch) = use_store::<MediaStore>();
    let mic_callback: Callback<String> = {
        let dispatch = dispatch.clone();
        Callback::from(move |audio| {
            dispatch.apply(ClientMediaMsg::AudioDeviceInit(audio))
        })
    };
    let cam_callback = {
        let dispatch = dispatch.clone();
        Callback::from(move |video| {
            dispatch.apply(ClientMediaMsg::VideoDeviceInit(video));
        })
    };
    html! {
        <>
            <DeviceSelector on_microphone_select={mic_callback} on_camera_select={cam_callback}/>
        </>
    }
}
