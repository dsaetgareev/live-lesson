use yew::{html, Html, function_component, use_state};

use crate::components::multi::client::welcome::WelcomeClient;
use crate::components::multi::host::welcome::WelcomeHost;
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
                false
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

