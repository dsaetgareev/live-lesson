
use wasm_peers::{get_random_session_id, SessionId};
use yew::{Component, Context, html, Html};
use log::error;

use crate::client::Client;
use crate::host::Host;
use crate::utils::global_window;
use crate::utils;

pub struct Multi {
    session_id: SessionId,
    is_host: bool,
}

impl Component for Multi {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let query_params = utils::get_query_params_multi();
        let (session_id, is_host) =
            match (query_params.get("session_id"), query_params.get("is_host").or(Some("session_id".to_owned()))) {
                (Some(session_string), Some(is_host)) => {
                    (SessionId::new(session_string), is_host == "true")
                }
                _ => {
                    let location = global_window().location();
                    let generated_session_id = get_random_session_id();
                    query_params.append("session_id", generated_session_id.as_str());
                    // query_params.append("host", "true");
                    let search: String = query_params.to_string().into();
                    if let Err(error) = location.set_search(&search) {
                        error!("Error while setting URL: {error:?}")
                    }
                    (generated_session_id, true)
                }
            };
        Self {
            is_host,
            session_id,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        false
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        
        html! {
            <main class="px-3">
                <p class="lead"> { "Share session id: " } <span class="line">{ &self.session_id }</span> </p>
                <p class="lead"> { "or just copy the page url." } </p>
                if self.is_host {
                    <Host />
                } else {
                    <Client />
                }
            </main>
        }
    }
}
