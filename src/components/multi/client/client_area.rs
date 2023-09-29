use std::{cell::RefCell, rc::Rc};

use monaco::api::TextModel;
use wasm_bindgen::JsCast;
use web_sys::{HtmlTextAreaElement, InputEvent, MouseEvent};
use yew::{Properties, Callback, Component, html};
use yew_icons::{Icon, IconId};

use crate::{models::{client::ClientProps, commons::AreaKind}, components::editor::editor::EditorWrapper, utils::inputs::ClientMessage};

const TEXTAREA_ID_CLIENT: &str = "client-textarea";

pub enum Msg {
    UpdateValue(String),
    Tick,
    SwitchArea(AreaKind),
}

#[derive(PartialEq, Properties)]
pub struct ClientAreaProps {
    pub client_props: Rc<RefCell<ClientProps>>,
    pub send_message_to_host_cb: Callback<ClientMessage>,
}

pub struct ClientArea {
    pub client_props: Rc<RefCell<ClientProps>>,
    pub send_message_to_host_cb: Callback<ClientMessage>,
}

impl ClientArea {
    pub fn send_message_to_host(&self, message: ClientMessage) {
        self.send_message_to_host_cb.emit(message);
    }
}

impl Component for ClientArea {
    type Message = Msg;

    type Properties = ClientAreaProps;

    fn create(ctx: &yew::Context<Self>) -> Self {
        Self { 
            client_props: ctx.props().client_props.clone(),
            send_message_to_host_cb: ctx.props().send_message_to_host_cb.clone(),
         }
    }

    fn update(&mut self, _ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::UpdateValue(content) => {
                let host_area_kind = self.client_props.borrow().client_area_kind;
                match host_area_kind {
                    AreaKind::Editor => {
                        self.client_props.clone().borrow_mut().client_editor_content = content.clone();
                    },
                    AreaKind::TextArea => {
                        self.client_props.clone().borrow_mut().client_text_area.set_content(content.clone());
                    },
                }                

                let message = ClientMessage::ClientToClient {
                             message: content,
                             area_kind: self.client_props.as_ref().borrow().client_area_kind
                };
                let _ = self.send_message_to_host(message);
                false
            },
            Msg::Tick => {
                true
            },
            Msg::SwitchArea(area_kind) => {
                let message = ClientMessage::ClientSwitchArea { message: area_kind };
                self.send_message_to_host(message);
                true
            },
        }
    }

    fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
        let text_model = TextModel::create(&ctx.props().client_props.borrow().client_editor_content, Some("java"), None).unwrap();
        // let on_host_editor_cb = &ctx.props().on_host_editor_cb.clone();
        let on_host_editor_cb = ctx.link().callback(|content: String| Msg::UpdateValue(content));

        let render = || {
            match &ctx.props().client_props.clone().borrow().client_area_kind {
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
                    let value = ctx.props().client_props.borrow().client_text_area.content.clone();

                    html! {
                        <div class="col document">
                            <textarea id={ TEXTAREA_ID_CLIENT } value={ value } { oninput } class="document" cols="100" rows="30" />
                        </div>
                    }
                },
            }
        };

        let render_batton_bar = || {
            let host_props = ctx.props().client_props.clone();
            let editor_click = ctx.link().callback(move |_: MouseEvent| {
                host_props.borrow_mut().client_area_kind = AreaKind::Editor;
                Msg::SwitchArea(AreaKind::Editor)
            });
            let host_props = ctx.props().client_props.clone();
            let text_area_click = ctx.link().callback(move |_: MouseEvent| {
                host_props.borrow_mut().client_area_kind = AreaKind::TextArea;
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
                <div class="col-3">
                    { render_batton_bar() }
                    { render() }
                </div>
                
            </>
        }
    }
}