use std::{rc::Rc, cell::RefCell};

use monaco::api::TextModel;
use yew::{Component, Properties, Callback, html};

use crate::{models::{commons::AreaKind, host::HostPorps}, components::editor::editor::EditorWrapper};

const TEXTAREA_ID: &str = "document-textarea";

pub enum Msg {
    UpdateValue(String),
    Tick,
    SwitchArea(AreaKind),
}

#[derive(PartialEq, Properties)]
pub struct HostAreaProps {
    pub host_props: Rc<RefCell<HostPorps>>,
    pub area_kind: AreaKind, // costy'l
    pub editor_content: String,
    pub text_area_content: String,
}


pub struct HostArea {
    pub host_props: Rc<RefCell<HostPorps>>,
}

impl Component for HostArea {
    type Message = ();

    type Properties = HostAreaProps;

    fn create(ctx: &yew::Context<Self>) -> Self {
        Self { 
            host_props: ctx.props().host_props.clone()
        }
    }

    fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
        let text_model = TextModel::create(&ctx.props().editor_content, Some("java"), None).unwrap();
        let on_host_editor_cb = Callback::from(|_content: String| log::info!("jkdjf"));

        let render = || {
            match self.host_props.clone().borrow().host_area_kind {
                AreaKind::Editor => {
                    html! {
                        <div class="col document">
                            <EditorWrapper on_cb={ on_host_editor_cb.clone() } text_model={ text_model.clone() } is_write={ true }/>
                        </div>
                    }
                },
                AreaKind::TextArea => {
                    
                    let value = ctx.props().text_area_content.clone();

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
                <div class="col">
                    { render() }
                    <div id="host-host">

                    </div>
                </div>
                
            </>
        }
    }
}