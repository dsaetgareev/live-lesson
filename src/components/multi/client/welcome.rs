use std::{cell::RefCell, rc::Rc, str::FromStr};

use wasm_peers::{SessionId, get_random_session_id};
use web_sys::MouseEvent;
use yew::{Component, html, use_state, function_component, Html};
use yewdux::prelude::use_store;

use crate::{models::audio::Audio, components::multi::client::client::Client, utils::{device::create_audio_decoder, self}, stores::client_store::{ClientStore, ClientMsg}};


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
        let to_host = to_client.clone();
        let session_id = session_id.clone();
        let dispatch = dispatch.clone();
        move |_e: MouseEvent| {
            dispatch.apply(ClientMsg::Init(*session_id));
            to_host.set(true);
        }
    };
    html! {
        if *to_client {
            <Client />
            
        } else {
            <button onclick={ on_init }>
                { 
                    "Заходи дорогой!"
                }
                
            </button>
        }
    }
}

pub enum Msg {
    ToClient(Rc<RefCell<Audio>>)
}

pub struct Welcome {
    pub is_client: bool,
    audio: Option<Rc<RefCell<Audio>>>,
}

impl Component for Welcome {
    type Message = Msg;

    type Properties = ();

    fn create(_ctx: &yew::Context<Self>) -> Self {
        Self { 
            is_client: false,
            audio: None,
        }
    }

    fn update(&mut self, _ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ToClient(audio) => {
                self.audio = Some(audio);
                self.is_client = true;
                true
            }
        }
    }

    fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
        let on_init = ctx.link().callback(|_| {
            let audio = Rc::new(RefCell::new(create_audio_decoder()));
            Msg::ToClient(audio)
        });
        
        html! {
            if !self.is_client {
                <button onclick={ on_init }>
                    { 
                        "Заходи дорогой!"
                    }
                    
                </button>
            } else {
                <Client />
            }
            
        }
    }
}