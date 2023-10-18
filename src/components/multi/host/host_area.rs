
use std::borrow::BorrowMut;

use monaco::api::TextModel;
use web_sys::{InputEvent, MouseEvent};
use yew::{html, Callback, Html, function_component};
use yew_icons::{Icon, IconId};
use yewdux::prelude::{use_store, Dispatch};

use crate::{models::commons::AreaKind, components::{editor::editor::EditorWrapper, multi::draw::paint::PaintF}, stores::{host_props_store::{HostHostMsg, HostPropsStore}, host_store::{self, HostStore}}, utils::inputs::Message};


const TEXTAREA_ID: &str = "document-textarea";


#[function_component(HostArea)]
pub fn host_area() -> Html {
    let (state, dispatch) = use_store::<HostPropsStore>();
    let global_dispatch = Dispatch::<HostStore>::new();
    let render = || {
        let area_kind = state.get_host_props().host_area_kind;
        match area_kind {
            AreaKind::Editor => {
                let on_host_editor_cb = {
                    let dispatch = dispatch.clone();
                    Callback::from(move |content: String| dispatch.apply(HostHostMsg::HostUpdateValue(content)))
                };
                let text_model = TextModel::create(&state.get_host_props().host_editor_content, Some("java"), None).unwrap();
                html! {
                    <div class="document">
                        <EditorWrapper on_cb={ on_host_editor_cb.clone() } text_model={ text_model.clone() } is_write={ false }/>
                    </div>
                }
            },
            AreaKind::TextArea => {
                let oninput = {
                    let dispatch = dispatch.clone();
                    move |e: InputEvent| {
                        dispatch.apply(HostHostMsg::HostTextAreaInput(e));
                    }
                };
                let value = state.get_host_props().host_area_content.content.clone();
                html! {
                    <div class="document">
                        <textarea id={ TEXTAREA_ID } value={ value } { oninput } class="document" cols="100" rows="30" />
                    </div>
                }
            },
        }
    };

    let render_paints = || {
        let send_message_all_cb = {
            let global_dispatch = global_dispatch.clone();
            Callback::from(move |message: Message| {
                global_dispatch.apply(host_store::Msg::SendMessage(message));
            })
        };
        let out = state.get_paints_f()
            .values()
            .map(|content| {
                html! {
                    <>
                        <PaintF
                            content={ content.clone() }
                            send_message_all_cb={send_message_all_cb.clone()}
                            is_host={ true }

                        />
                    </>
                }
            })
            .collect::<Html>();
        out
    };


    html! {
        <>
            <div class="host-box">
                <HostButtonBar />
                <div class="host-content-box">
                    { render() }
                    <div id="host-paint" class="host-paint">
                        { render_paints() }
                    </div>
                </div>
                
            </div>
            
        </>
    }
}



#[function_component(HostButtonBar)]
pub fn host_button_bar() -> Html {
    let (state, dispatch) = use_store::<HostPropsStore>();

    let editor_click = {
        let dispatch = dispatch.clone();
        move |_e: MouseEvent| {
            dispatch.apply(HostHostMsg::SwitchHostArea(AreaKind::Editor));
        }
    };
    let text_area_click = {
        let dispatch = dispatch.clone();
        move |_e: MouseEvent| {
            dispatch.apply(HostHostMsg::SwitchHostArea(AreaKind::TextArea));
        }
    };
    let paint_click = {
        let dispatch = dispatch.clone();
        move |_e: MouseEvent| {
            dispatch.apply(HostHostMsg::OpenPaint);
        }
    };
    let on_communication = {
        let dispatch = dispatch.clone();
        move |_e: MouseEvent| {
            dispatch.apply(HostHostMsg::OnCummunication);
        }
    };

    html! {
        <>
            <button>
                <Icon icon_id={IconId::FontAwesomeSolidCode} onclick={ editor_click }/>
            </button>
            <button>
                <Icon icon_id={IconId::BootstrapFileEarmarkText} onclick={ text_area_click }/>
            </button>
            <button>
                <Icon icon_id={IconId::HeroiconsSolidPaintBrush} onclick={ paint_click }/>
            </button>
            <button onclick={ on_communication }>
                { 
                    if state.get_host_props().is_communication.clone() {
                        html! { <Icon icon_id={IconId::BootstrapPeopleFill}/> }
                    } else {
                        html! { <Icon icon_id={IconId::BootstrapPeople}/> }
                    }
                }
            </button>
        </>
    }
}
