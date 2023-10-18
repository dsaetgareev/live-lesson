use monaco::api::TextModel;
use yew::{Callback, html, Html, function_component};
use yew_icons::{Icon, IconId};
use yewdux::prelude::use_store;

use crate::{models::commons::AreaKind, components::editor::editor::EditorWrapper, stores::client_props_store::{ClientPropsStore, ClientPropsMsg}};

const TEXTAREA_ID_CLIENT: &str = "client-textarea";


#[function_component(ClientArea)]
pub fn client_area() -> Html {

    let (state, dispatch) = use_store::<ClientPropsStore>();

    let render = || {
        match state.get_client_props().client_area_kind {
            AreaKind::Editor => {
                let text_model = TextModel::create(&state.get_client_props().client_editor_content, Some("java"), None).unwrap();
                let on_host_editor_cb = {
                    let dispatch = dispatch.clone();
                    Callback::from(move |content| {
                        dispatch.apply(ClientPropsMsg::UpdateClientValue(content));
                    })
                };
                let is_write = &state.get_client_props().is_write;
                html! {
                    <div class="col document">
                        <EditorWrapper on_cb={ on_host_editor_cb.clone() } text_model={ text_model.clone() } is_write={ is_write }/>
                    </div>
                }
            },
            AreaKind::TextArea => {
                let oninput =  {
                    let dispatch = dispatch.clone();
                    Callback::from(move |e| dispatch.apply(ClientPropsMsg::UpdateClientTextArea(e)))
                };
                let value = state.get_client_props().client_text_area.content.clone();
                html! {
                    <div class="col document">
                        <textarea id={ TEXTAREA_ID_CLIENT } value={ value } { oninput } class="document" cols="100" rows="30" />
                    </div>
                }
            },
        }
    };

    html! {
       <div class="col-3">
            <ClientButtonBar />
            { render() }
        </div>
    }
}

#[function_component(ClientButtonBar)]
pub fn client_button_bar() -> Html {

    let (_state, dispatch) = use_store::<ClientPropsStore>();

    let editor_click = {
        let dispatch = dispatch.clone();
        Callback::from(move |_| {
            dispatch.apply(ClientPropsMsg::SwitchArea(AreaKind::Editor));
        })
    };
    let text_area_click = {
        let dispatch = dispatch.clone();
        Callback::from(move |_| {
            dispatch.apply(ClientPropsMsg::SwitchArea(AreaKind::TextArea));
        })
    };

    html! {
        <>
            <button>
                <Icon icon_id={IconId::FontAwesomeSolidCode} onclick={ editor_click }/>
            </button>
            <button>
                <Icon icon_id={IconId::BootstrapFileEarmarkText} onclick={ text_area_click }/>
            </button>
        </>
    }
}