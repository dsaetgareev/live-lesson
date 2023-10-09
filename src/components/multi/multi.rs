use wasm_peers::get_random_session_id;
use yew::{Component, Context, html, Html};
use log::error;

use crate::components::multi::client::welcome::Welcome;
use crate::components::multi::host::welcome::WelcomeHost;
use crate::utils::dom::global_window;
use crate::utils;

pub struct Multi {
    is_host: bool,
}

impl Component for Multi {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let query_params = utils::dom::get_query_params_multi();
        let is_host =
            match query_params.get("is_host").or(Some("session_id".to_owned())) {
                Some(is_host) => {
                    is_host == "true"
                }
                _ => {
                    let location = global_window().location();
                    let generated_session_id = get_random_session_id();
                    query_params.append("session_id", &uuid::Uuid::from_u128(generated_session_id.inner()).to_string());
                    // query_params.append("host", "true");
                    let search: String = query_params.to_string().into();
                    if let Err(error) = location.set_search(&search) {
                        error!("Error while setting URL: {error:?}")
                    }
                    true
                }
            };
        Self {
            is_host,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        false
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        
        html! {
            <div class="main">
                if self.is_host {
                    <WelcomeHost />
                } else {
                    <Welcome />
                }
            </div>
        }
    }
}
