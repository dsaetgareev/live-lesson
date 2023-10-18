use web_sys::MouseEvent;
use yew::{Properties, html, Html, Callback, function_component, use_effect};
use yew_icons::{Icon, IconId};
use yewdux::prelude::use_store;

use crate::{utils::dom::{create_video_id, get_element}, models::commons::AreaKind, stores::client_items_store::{ClientItemsStore, ClientItemMsg}};


#[derive(Properties, PartialEq)]
pub struct ItemPorps {
    pub key_id: String,
    pub value: String,
}


#[function_component(ClientBox)]
pub fn client_box(props: &ItemPorps) -> Html {
    let (_state, dispatch) = use_store::<ClientItemsStore>();
    let key = props.key_id.clone();
    let value = props.value.clone();
    let client_id = key.clone();
    let client_logo_id = create_video_id(format!("{}_{}", "client-video-logo", key.clone()));    
    let box_id = format!("item-box-{}", key.clone());

    use_effect({
        let box_id = box_id.clone();
        let client_id = client_id.clone();
        move || {
            let box_div = get_element(&box_id).unwrap();
            let video = get_element(&create_video_id(client_id)).unwrap();
            let _ = box_div.append_child(&video);
         }
    });

    let item_click = {
        let dispatch = dispatch.clone();
        move |e: MouseEvent| {
            dispatch.apply(ClientItemMsg::ChooseItem(e));
        }
    };
    let on_switch_video = {
        let dispatch = dispatch.clone();
        let video_switch_id = client_id.clone();
        Callback::from(move |_| {
            dispatch.apply(ClientItemMsg::SwitchVideo(video_switch_id.clone()));
        })
    };
    let on_switch_speakers = {
        let dispatch = dispatch.clone();
        let speakers_id = client_id.clone();
        Callback::from(move |_| {
            dispatch.apply(ClientItemMsg::SwitchSpeakers(speakers_id.clone()));
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

    let (state, _dispatch) = use_store::<ClientItemsStore>();

    let render = || {
        log::error!("clent items");
        let players = state.get_players();

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