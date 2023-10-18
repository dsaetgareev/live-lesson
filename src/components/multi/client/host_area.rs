use monaco::api::TextModel;
use yew::{Callback, html, Html, function_component};
use yewdux::prelude::use_store;

use crate::{models::commons::AreaKind, components::editor::editor::EditorWrapper, stores::host_props_store::HostPropsStore};

const TEXTAREA_ID: &str = "document-textarea";

#[function_component(HostArea)]
pub fn host_area() -> Html {
    let (state, _dispatch) = use_store::<HostPropsStore>();

    let render = || {
        match state.get_host_props().host_area_kind {
            AreaKind::Editor => {
                let text_model = TextModel::create(&state.get_host_props().host_editor_content, Some("java"), None).unwrap();
                let on_host_editor_cb = Callback::default();
                html! {
                    <div class="col document">
                        <EditorWrapper on_cb={ on_host_editor_cb.clone() } text_model={ text_model.clone() } is_write={ true }/>
                    </div>
                }
            },
            AreaKind::TextArea => {
                
                let value = state.get_host_props().host_area_content.content.clone();
                html! {
                    <div class="col document">
                        <textarea id={ TEXTAREA_ID } value={ value } class="document" cols="100" rows="30" />
                    </div>
                }
            },
        }
    };
   
    html! {
        <>
            <div class="host-content-box">
                { render() }
                <div id="host-paint" class="host-paint">
                </div>
            </div>
            
        </>
    }    
}