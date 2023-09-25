use std::{cell::RefCell, rc::Rc};

use monaco::api::TextModel;
use yew::{Component, Properties, html, Callback};

use crate::{models::client::ClientProps, components::editor::editor::EditorWrapper};

#[derive(PartialEq, Properties)]
pub struct ClientAreaProps {
    pub client_props: Rc<RefCell<ClientProps>>,
    pub on_client_editor_cb: Callback<String>,
}

pub struct ClientArea {

}

impl Component for ClientArea {
    type Message = ();

    type Properties = ClientAreaProps;

    fn create(_ctx: &yew::Context<Self>) -> Self {
        Self {  }
    }

    fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
        let text_model_client = TextModel::create(&ctx.props().client_props.borrow().client_content, Some("java"), None).unwrap();
        let on_client_editor_cb = &ctx.props().on_client_editor_cb;
    
        html! {
            <div class="col document">
                <EditorWrapper on_cb={ on_client_editor_cb } text_model={ text_model_client } is_write={ &ctx.props().client_props.borrow().is_write }/>
            </div>
        }
    }
}