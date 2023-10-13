use serde::{Deserialize, Serialize};
use wasm_peers::get_random_session_id;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::components::document::document::Query;
use crate::Route;

#[derive(Serialize, Deserialize)]
pub struct GameQuery {
    pub session_id: String,
    pub is_host: bool,
}

impl GameQuery {
    pub(crate) fn new(session_id: String, is_host: bool) -> Self {
        GameQuery {
            session_id,
            is_host,
        }
    }
}

pub enum Msg {
    UpdateInput,
}

#[function_component(Home)]
pub fn home() -> Html {

    let session_id = use_state(|| String::default());

    let on_update_input = {
        let session_id = session_id.clone();
        Callback::from(move |e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let value = input.value().parse::<String>().unwrap_or(String::default());
            log::info!("value = {}", &*value);
            session_id.set(value);
        })
    };

    let navigator = use_navigator().unwrap();
    let start_as_host_text_document = {
        let session_id = session_id.clone();
        let navigator = navigator.clone();
        Callback::from(move |_| {
            navigator
                .push_with_query(
                    &Route::Document,
                    &Query::new((*session_id).clone()),
                )
                .unwrap();
        })
    };
    let start_as_host_table_document = {
        let session_id = session_id.clone();
        if session_id.is_empty() {
            session_id.set(uuid::Uuid::from_u128(get_random_session_id().inner()).to_string());
        }
        let navigator = navigator.clone();
        Callback::from(move |_| {
            navigator
                .push_with_query(
                    &Route::Table,
                    &Query::new((*session_id).clone()),
                )
                .unwrap();
        })
    };
    let start_as_host_multi_document = {
        let session_id = session_id.clone();
        if session_id.is_empty() {
            session_id.set(uuid::Uuid::from_u128(get_random_session_id().inner()).to_string());
        }
        let navigator = navigator.clone();
        Callback::from(move |_| {
            navigator
                .push_with_query(
                    &Route::Multi,
                    &GameQuery::new((*session_id).clone(), true),
                )
                .unwrap();
        })
    };

    html! {
        <div class="cover-container d-flex w-100 h-100 p-3 mx-auto flex-column">
            <header class="mb-auto">
                <div>
                    <h3 class="float-md-start mb-0">{ "Live Lesson"  }</h3>
                </div>
            </header>

            <main class="px-3">
                <hr />
                <p class="lead">{ "Создайте документ" }</p>
                <p class="lead">
                    <button onclick={ start_as_host_text_document } class="btn btn-lg btn-secondary fw-bold border-white">
                        { "Текстовый документ" }
                    </button>
                </p>
                <p class="lead">
                    <button onclick={ start_as_host_table_document } class="btn btn-lg btn-secondary fw-bold border-white">
                        { "Таблица" }
                    </button>
                </p>
                <p class="lead">
                    <button onclick={ start_as_host_multi_document } class="btn btn-lg btn-secondary fw-bold border-white">
                        { "Мультидокумент" }
                    </button>
                </p>
                <p class="lead">{ "или подключитесь к существующему документу" }</p>
                <p class="lead">
                    <input id="join-input"
                        placeholder={ "Session id " }
                        onchange={ on_update_input }
                    />
                </p>
                
            </main>

            <footer class="mt-auto text-white-50">
                <p>{ "Style based on Cover Bootstrap example." }</p>
            </footer>
        </div>
    }
}
