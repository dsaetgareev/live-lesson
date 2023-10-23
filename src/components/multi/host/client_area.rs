
use monaco::api::TextModel;
use wasm_bindgen::JsCast;
use web_sys::{InputEvent, HtmlTextAreaElement};
use yew::prelude::*;
use yewdux::prelude::*;

use crate::{models::commons::AreaKind, components::editor::editor::EditorWrapper, stores::client_props_store::{ClientPropsStore, HostClientMsg}};

const TEXTAREA_ID_CLIENT: &str = "client-textarea";

pub enum Msg {
    SendClient(String),
    Tick,
}

#[function_component(ClientArea)]
pub fn client_area() -> Html {

    let (state, dispatch) = use_store::<ClientPropsStore>();     

    let on_host_editor_cb = {
        let dispatch = dispatch.clone();
        Callback::from(move |content: String| dispatch.apply(HostClientMsg::HostClientToClient(content)))
    };

    let render = || {
        let area_kind = state.get_client_props().client_area_kind;
        match area_kind {
            AreaKind::Editor => {
                let text_model = TextModel::create(&state.get_client_props().client_editor_content, Some("java"), None).unwrap();
       
                let is_write = state.get_client_props().is_write;
                html! {
                    <div class="document">
                        <EditorWrapper on_cb={ on_host_editor_cb.clone() } text_model={ text_model.clone() } is_write={ is_write }/>
                    </div>
                }
            },
            AreaKind::TextArea => {
                let on_host_editor_cb = on_host_editor_cb.clone();
                let oninput = Callback::from(move |e: InputEvent| {
                    let content = e
                        .target()
                        .unwrap()
                        .unchecked_into::<HtmlTextAreaElement>()
                        .value();
                    on_host_editor_cb.emit(content);
                });
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
        <>
            {render()}
        </>
    }
}
