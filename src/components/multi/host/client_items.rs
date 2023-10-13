use std::{rc::Rc, cell::RefCell, collections::HashMap};

use wasm_bindgen::JsCast;
use wasm_peers::UserId;
use web_sys::{HtmlElement, MouseEvent};
use yew::{Component, Properties, html, Html, Callback, function_component};
use yew_icons::{Icon, IconId};
use yewdux::prelude::use_store;

use crate::{utils::dom::{create_video_id, get_element}, models::{client::ClientItem, commons::AreaKind}, stores::host_store::{HostStore, self}, components::multi::host::client_items::_ItemPorps::key_id};


#[derive(Properties, PartialEq)]
pub struct ItemPorps {
    pub key_id: String,
    pub value: String,
}


#[function_component(ClientBox)]
pub fn client_box(props: &ItemPorps) -> Html {
    let (_state, dispatch) = use_store::<HostStore>();
    let key = props.key_id.clone();
    let value = props.value.clone();
    let client_id = key.clone();
    let client_logo_id = create_video_id(format!("{}_{}", "client-video-logo", key.clone()));  
    
    let box_id = format!("item-box-{}", key.clone());
    let item_click = {
        let dispatch = dispatch.clone();
        move |e: MouseEvent| {
            dispatch.apply(host_store::Msg::ChooseItem(e));
        }
    };
    let on_switch_video = {
        let dispatch = dispatch.clone();
        let video_switch_id = client_id.clone();
        Callback::from(move |_| {
            dispatch.apply(host_store::Msg::SwitchVideo(video_switch_id.clone()));
        })
    };
    let on_switch_speakers = {
        let dispatch = dispatch.clone();
        let speakers_id = client_id.clone();
        Callback::from(move |_| {
            dispatch.apply(host_store::Msg::SwitchSpeakers(speakers_id.clone()));
        })
    };
    html! {
        <>
            <div class="item-box">
                <div id={ box_id } client_id={ client_id.clone() } class="col" onclick={ item_click.clone() }>
                    <textarea id={ key } client_id={ client_id.clone() } value={ value } class="doc-item" cols="100" rows="30" />
                    // <video id={ video_id } client_id={ client_id.clone() } autoplay=true class="item-canvas"></video>
                    <div class="col">
                        <button onclick={ on_switch_video } client_id={ client_id.clone() } >{"video ->"}</button>
                        <button onclick={ on_switch_speakers } client_id={ client_id.clone() }>{"audio ->"}</button>
                    </div>
                    // <canvas id={ video_id } client_id={ client_id } class="item-canvas vis" ></canvas>
                    <div id={ client_logo_id } class="unvis">
                        <Icon icon_id={IconId::FontAwesomeSolidHorseHead}/>
                    </div>
                </div>
                
            </div>
            
        </>
    }
}

#[function_component(ClientItems)]
pub fn client_items() -> Html {

    let (state, _dispatch) = use_store::<HostStore>();

    let render = || {
        log::error!("clent items");
        let players = &state
            .players;

        players
            .clone()
            .into_keys()
            .map(|key| {
            match players.get(&key) {
                Some(client_item) => {
                    match client_item.area_kind {
                        AreaKind::Editor => {
                            html! {
                                <>
                                    <ClientBox key_id={ key.to_string() } value={ client_item.editor_content.clone() } />
                                </>
                            }
                        },
                        AreaKind::TextArea => {
                            html! {
                                <>
                                    <ClientBox key_id={ key.to_string() } value={ client_item.text_area_content.clone() } />
                                </>
                            }
                            
                        },
                    }
                },
                None => {
                    html! {
                        <>
                        </>
                    }
                },
            }
            
            
        }).collect::<Html>()      
    };

    html! {
        <>
            { render() }
        </>
    }
}