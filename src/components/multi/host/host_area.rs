use std::{rc::Rc, cell::RefCell};

use monaco::api::TextModel;
use wasm_bindgen::JsCast;
use wasm_peers::UserId;
use web_sys::{InputEvent, HtmlTextAreaElement, MouseEvent};
use yew::{Component, Properties, html, Callback};
use yew_icons::{Icon, IconId};

use crate::{models::{host::HostPorps, commons::AreaKind}, components::editor::editor::EditorWrapper, utils::{self, inputs::Message}};


const TEXTAREA_ID: &str = "document-textarea";

pub enum Msg {
    UpdateValue,
    Tick,
    SwitchArea(AreaKind),
}

#[derive(PartialEq, Properties)]
pub struct HostAreaProps {
    pub host_props: Rc<RefCell<HostPorps>>,
    pub on_host_editor_cb: Callback<String>,
    pub send_message_cb: Callback<(UserId, String)>,
    pub send_message_all_cb: Callback<String>,
}

pub struct HostArea {

}

impl Component for HostArea {
    type Message = Msg;

    type Properties = HostAreaProps;

    fn create(_ctx: &yew::Context<Self>) -> Self {
        Self {  }
    }

    fn update(&mut self, ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::UpdateValue => {
                true
            },
            Msg::Tick => {
                true
            },
            Msg::SwitchArea(area_kind) => {
                let message = Message::HostSwicthArea { message: area_kind };
                match serde_json::to_string(&message) {
                    Ok(message) => {
                        ctx.props().send_message_all_cb.emit(message);
                    },
                    Err(_) => todo!(),
                }
                true
            }
        }
    }

    fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {

        let text_model = TextModel::create(&ctx.props().host_props.borrow().host_editor_content, Some("java"), None).unwrap();
        let on_host_editor_cb = &ctx.props().on_host_editor_cb.clone();

        let render = || {
            match &ctx.props().host_props.clone().borrow().host_area_kind {
                AreaKind::Editor => {
                    html! {
                        <div class="col document">
                            <EditorWrapper on_cb={ on_host_editor_cb.clone() } text_model={ text_model.clone() } is_write={ true }/>
                        </div>
                    }
                },
                AreaKind::TextArea => {
                    let on_host_editor_cb = on_host_editor_cb.clone();
                    let oninput = ctx.link().callback(move |e: InputEvent| {
                        let content = e
                            .target()
                            .unwrap()
                            .unchecked_into::<HtmlTextAreaElement>()
                            .value();
                        on_host_editor_cb.emit(content);
                        Msg::Tick
                    });
                    let value = ctx.props().host_props.borrow().host_area_content.content.clone();

                    html! {
                        <div class="col document">
                            <textarea id={ TEXTAREA_ID } value={ value } { oninput } class="document" cols="100" rows="30" />
                        </div>
                    }
                },
            }
        };

        let render_batton_bar = || {
            let host_props = ctx.props().host_props.clone();
            let editor_click = ctx.link().callback(move |_: MouseEvent| {
                host_props.borrow_mut().host_area_kind = AreaKind::Editor;
                Msg::SwitchArea(AreaKind::Editor)
            });
            let host_props = ctx.props().host_props.clone();
            let text_area_click = ctx.link().callback(move |_: MouseEvent| {
                host_props.borrow_mut().host_area_kind = AreaKind::TextArea;
                Msg::SwitchArea(AreaKind::TextArea)
            });

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
        };

       
        html! {
            <>
                <div class="col">
                    { render_batton_bar() }
                    { render() }
                </div>
                
            </>
        }
    }
}