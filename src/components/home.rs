use std::cell::RefCell;
use std::rc::Rc;

use serde::{Deserialize, Serialize};
use wasm_peers::get_random_session_id;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::components::document::document::Query;
use crate::models::audio::Audio;
use crate::utils::device::create_audio_decoder;
use crate::utils::dom::get_input;
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

pub struct Home {
    input: String,
}

impl Component for Home {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            input: String::new(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Self::Message::UpdateInput => match get_input("join-input") {
                Ok(input) => {
                    self.input = input.value();
                    true
                }
                Err(err) => {
                    eprintln!("failed to get input: {err}");
                    false
                }
            },
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let mut session_id = self.input.clone();
        if self.input.is_empty() {
            session_id = uuid::Uuid::from_u128(get_random_session_id().inner()).to_string();
        }
        let navigator = ctx.link().navigator().unwrap();
        let start_as_host_text_document = {
            let navigator = navigator.clone();
            Callback::from(move |_| {
                navigator
                    .push_with_query(
                        &Route::Document,
                        &Query::new(session_id.clone()),
                    )
                    .unwrap();
            })
        };
        let start_as_host_table_document = {
            let mut session_id = self.input.clone();
            if self.input.is_empty() {
                session_id = uuid::Uuid::from_u128(get_random_session_id().inner()).to_string();
            }
            let navigator = navigator.clone();
            Callback::from(move |_| {
                navigator
                    .push_with_query(
                        &Route::Table,
                        &Query::new(session_id.clone()),
                    )
                    .unwrap();
            })
        };
        let start_as_host_multi_document = {
            let mut session_id = self.input.clone();
            if self.input.is_empty() {
                session_id = uuid::Uuid::from_u128(get_random_session_id().inner()).to_string();
            }
            let navigator = navigator.clone();
            Callback::from(move |_| {
                navigator
                    .push_with_query(
                        &Route::Multi,
                        &GameQuery::new(session_id.clone(), true),
                    )
                    .unwrap();
            })
        };
        let update_input = ctx.link().callback(|_| Msg::UpdateInput);
        
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
                                oninput={ update_input }
                            />
                        </p>
                        
                    </main>

                    <footer class="mt-auto text-white-50">
                        <p>{ "Style based on Cover Bootstrap example." }</p>
                    </footer>
                </div>
        }
    }
}
