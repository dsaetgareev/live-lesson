use serde::{Deserialize, Serialize};
use wasm_peers::get_random_session_id;
use yew::prelude::*;
use yew_router::prelude::*;

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

    let navigator = use_navigator().unwrap();
    
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
                
                <p class="lead">
                    <button onclick={ start_as_host_multi_document } class="btn btn-lg btn-secondary fw-bold border-white">
                        { "Создать встречу" }
                    </button>
                </p>
               
                
            </main>

            <footer class="mt-auto text-white-50">
                <p>{ "" }</p>
            </footer>
        </div>
    }
}
