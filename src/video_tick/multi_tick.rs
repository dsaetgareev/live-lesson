
use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::Closure;
use wasm_peers::{get_random_session_id, ConnectionType, SessionId, UserId};
use yew::{html, Component, Context, Html};
use log::error;

use crate::utils::{get_window, global_window};
use crate::utils;

use super::client::ClientGame;
use super::footballers_game::{FootballersGame, Game};
use super::host::HostGame;
use super::models::Player;

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


pub enum GameMsg {
    CopyLink,
    Init,
    Tick,
}

pub struct MultiTick {
    session_id: SessionId,
    is_host: bool,
    game: Option<FootballersGame>,
    tick_callback: Closure<dyn FnMut()>,
    players: HashMap<UserId, Player>, 
}

const TEXTAREA_ID: &str = "host-textarea";
const TEXTAREA_ID_CLIENT: &str = "client-textarea";

impl Component for MultiTick {
    type Message = GameMsg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let query_params = utils::get_query_params_multi();
        let (session_id, is_host) =
            match (query_params.get("session_id"), query_params.get("is_host")) {
                (Some(session_string), Some(is_host)) => {
                    (SessionId::new(session_string), is_host == "true")
                }
                _ => {
                    let location = global_window().location();
                    let generated_session_id = get_random_session_id();
                    query_params.append("session_id", generated_session_id.as_str());
                    query_params.append("host", "true");
                    let search: String = query_params.to_string().into();
                    if let Err(error) = location.set_search(&search) {
                        error!("Error while setting URL: {error:?}")
                    }
                    (generated_session_id, true)
                }
            };
        let tick_callback = {
            let link = ctx.link().clone();
            Closure::wrap(Box::new(move || link.send_message(GameMsg::Tick)) as Box<dyn FnMut()>)
        };
        ctx.link().send_message(GameMsg::Init);
        Self {
            is_host,
            session_id,
            game: None,
            tick_callback,
            players: HashMap::default(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            GameMsg::CopyLink => {
                false
            }
            GameMsg::Init => {
                self.game = Some(init_game(
                    self.is_host,
                    self.session_id.clone(),
                ));
                ctx.link().send_message(GameMsg::Tick);
                false
            }
            GameMsg::Tick => {
                match self.game.as_mut() {
                    
                    Some(game) => {
                        game.tick();
                        self.players = game.get_players();
                        log::info!("players {}", self.players.len());
                        if !game.ended() {
                            if let Err(error) = get_window().unwrap().request_animation_frame(
                                self.tick_callback.as_ref().unchecked_ref(),
                            ) {
                                error!("Failed requesting next animation frame: {error:?}");
                            }
                        }
                    }
                    None => {
                        error!("No initialized game object yet.");
                    }
                }
                true
            }
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let disabled = false;
        let placeholder = "This is a live document shared with other users.\nYou will be allowed \
                           to write once other join, or your connection is established.";

        
        let render_item = |key: String, value: String| {
            html! {
                    <>
                        <div>
                            <textarea id={ key } value={ value } class="doc-item" cols="100" rows="30" { disabled } { placeholder } />
                        </div>
                    </>
            }
        };
       
        let render_row = || {
            self.players.clone()
                .into_keys()
                .map(|key| {
                    let (key, value) = self.players.get_key_value(&key).unwrap();
                    render_item(key.to_string(), value.current_input.value.clone())
                })
        };

        


        // let onclick = ctx.link().callback(|_| GameMsg::CopyLink);

        // let oninput = ctx.link().callback(|_| Self::Message::UpdateValue);
        
        html! {
            <main class="px-3">

                <p class="lead"> { "Share session id: " } <span class="line">{ &self.session_id }</span> </p>
                <p class="lead"> { "or just copy the page url." } </p>
                <div class="row">
                    { for render_row() }
                </div>
                <textarea id={ TEXTAREA_ID_CLIENT } class="doc-item" cols="100" rows="30" { disabled } { placeholder } />
                <textarea id={ TEXTAREA_ID } class="document" cols="100" rows="30" { disabled } { placeholder } />
            </main>
        }
    }
}


fn init_game(is_host: bool, session_id: SessionId) -> FootballersGame {

    let connection_type = ConnectionType::StunAndTurn {
        stun_urls: env!("STUN_SERVER_URLS").to_string(),
        turn_urls: env!("TURN_SERVER_URLS").to_string(),
        username: env!("TURN_SERVER_USERNAME").to_string(),
        credential: env!("TURN_SERVER_CREDENTIAL").to_string(),
    };
    let signaling_server_url = concat!(env!("SIGNALING_SERVER_URL"), "/one-to-many");
    let mut game = if is_host {
        FootballersGame::Host(HostGame::new(
            session_id,
            connection_type,
            signaling_server_url,
        ))
    } else {
        FootballersGame::Client(ClientGame::new(
            session_id,
            connection_type,
            signaling_server_url,
        ))
    };
    game.init();
    game
}
