use wasm_peers::get_random_session_id;
use yew::{html, Html, function_component, use_state};
use log::error;

use crate::components::multi::client::welcome::WelcomeClient;
use crate::components::multi::host::welcome::WelcomeHost;
use crate::utils::dom::global_window;
use crate::utils;

#[function_component(Multi)]
pub fn multi() -> Html {

    let is_host = use_state(|| {
        let query_params = utils::dom::get_query_params_multi();
        match query_params.get("is_host").or(Some("session_id".to_owned())) {
            Some(is_host) => {
                is_host == "true"
            }
            _ => {
                let location = global_window().location();
                let generated_session_id = get_random_session_id();
                query_params.append("session_id", &uuid::Uuid::from_u128(generated_session_id.inner()).to_string());
                let search: String = query_params.to_string().into();
                if let Err(error) = location.set_search(&search) {
                    error!("Error while setting URL: {error:?}")
                }
                true
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

