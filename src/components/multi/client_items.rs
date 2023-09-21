use std::{rc::Rc, cell::RefCell, collections::HashMap};

use wasm_bindgen::JsCast;
use wasm_peers::UserId;
use web_sys::{HtmlElement, MouseEvent};
use yew::{Component, Properties, html, Html, Callback};

use crate::utils::dom::create_video_id;

pub enum Msg {
    ChooseItem(String),
    SwitchSpeakers(String),
    SwitchVideo(String),
}

#[derive(PartialEq, Properties)]
pub struct ClientItemsPorps {
    pub players: Rc<RefCell<HashMap<UserId, String>>>,
    pub on_switch_speakers: Callback<String>,
    pub on_switch_video: Callback<String>,
    pub on_choose_item: Callback<String>,
}


pub struct ClientItems {
    pub players: Rc<RefCell<HashMap<UserId, String>>>,
}

impl Component for ClientItems {
    type Message = Msg;

    type Properties = ClientItemsPorps;

    fn create(ctx: &yew::Context<Self>) -> Self {
        let players = ctx.props().players.clone();
        Self { players: players }
    }

    fn update(&mut self, ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ChooseItem(client_id) => {
                ctx.props().on_choose_item.emit(client_id);
                false
            },
            Msg::SwitchSpeakers(speakers_id) => {
                ctx.props().on_switch_speakers.emit(speakers_id);
                false
            },
            Msg::SwitchVideo(video_switch_id) => {
                ctx.props().on_switch_video.emit(video_switch_id);
                false
            },
        }
    }

    fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
        let item_click = ctx.link().callback(|e: MouseEvent| {
            let target: HtmlElement = e
                .target()
                .unwrap()
                .dyn_into()
                .unwrap();
            let client_id = target.get_attribute("client_id").unwrap();
            Msg::ChooseItem(client_id)

        });

        let render_item = |key: String, value: String| {
            let client_id = key.clone();
            let video_id = create_video_id(key.clone());
            let speakers_id = client_id.clone();
            let video_switch_id = client_id.clone();
            let on_switch_speakers = ctx.link().callback(move |_| Msg::SwitchSpeakers(speakers_id.clone()));
            let on_switch_video = ctx.link().callback(move |_|  Msg::SwitchVideo(video_switch_id.clone()));
            html! {
                    <>
                        <div class="col-sm">
                            <div client_id={ client_id.clone() } class="col" onclick={ item_click.clone() }>
                                <textarea id={ key } client_id={ client_id.clone() } value={ value } class="doc-item" cols="100" rows="30" />
                                // <video id={ video_id } client_id={ client_id.clone() } autoplay=true class="item-canvas"></video>
                                <div class="col">
                                    <button onclick={ on_switch_video } client_id={ client_id.clone() } >{"video ->"}</button>
                                    <button onclick={ on_switch_speakers } client_id={ client_id.clone() }>{"audio ->"}</button>
                                </div>
                                <canvas id={ video_id } client_id={ client_id } class="item-canvas" ></canvas>
                            </div>
                            
                        </div>
                        
                    </>
            }
        };

        let render = || {
            self.players.borrow().clone()
            .into_keys()
            .map(|key| {
                let value = String::from(self.players.borrow().get(&key).unwrap());
                // log::info!("value {}", value.clone());
                render_item(key.to_string(), value.to_string())
            }).collect::<Html>()      
        };

        html! {
            <div class="row">
                { render() }
            </div>
        }
    }
}