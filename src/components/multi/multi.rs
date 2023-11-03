use std::str::FromStr;

use wasm_peers::{get_random_session_id, SessionId};
use yew::{html, Html, function_component, use_state, use_effect};
use yewdux::prelude::Dispatch;

use crate::components::multi::client::welcome::WelcomeClient;
use crate::components::multi::host::welcome::WelcomeHost;
use crate::stores::client_store::{ClientStore, ClientMsg};
use crate::stores::host_store::{HostStore, self};
use crate::utils;

#[function_component(Multi)]
pub fn multi() -> Html {

    let session_id = use_state(|| {
        let query_params = utils::dom::get_query_params_multi();
        let session_id = match query_params.get("session_id") {
            Some(session_string) => {
                log::error!("from address");
                log::error!("session_string {}", session_string);
                SessionId::new(uuid::Uuid::from_str(&session_string).unwrap().as_u128())
            }
            _ => {
                log::error!("generated client");
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

    let is_host = use_state(|| {
        let query_params = utils::dom::get_query_params_multi();
        match query_params.get("is_host").or(Some("session_id".to_owned())) {
            Some(is_host) => {
                is_host == "true"
            }
            _ => {
                false
            }
        }
    });

    use_effect({
        let is_host = is_host.clone();
        let session_id = session_id.clone();
        log::error!("session_did {}", session_id.to_string());
        move || {
            if *is_host {
                let dispatch = Dispatch::<HostStore>::new();
                dispatch.apply(host_store::Msg::Init(*session_id));
            } else {
                let dispatch = Dispatch::<ClientStore>::new();
                dispatch.apply(ClientMsg::Init(*session_id));
            }
        }
    });

    html! {
        <div class="main">
            if *is_host {
                <WelcomeHost />
            } else {
                <WelcomeClient />
            }
        </div>
    }
}

