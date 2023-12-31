use std::{rc::Rc, cell::RefCell};

use monaco::api::TextModel;
use wasm_bindgen::JsCast;
use wasm_peers::UserId;
use web_sys::{InputEvent, HtmlTextAreaElement, MouseEvent};
use yew::{Component, Properties, html, Callback};
use yew_icons::{Icon, IconId};

use crate::{models::{host::HostPorps, commons::AreaKind}, components::{editor::editor::EditorWrapper, multi::draw::{self}}, utils::inputs::Message};


const TEXTAREA_ID: &str = "document-textarea";

pub enum Msg {
    UpdateValue(String),
    Tick,
    SwitchArea(AreaKind),
    OpenPaint,
    OnCummunication,
}

#[derive(PartialEq, Properties)]
pub struct HostAreaProps {
    pub host_props: Rc<RefCell<HostPorps>>,
    pub send_message_cb: Callback<(UserId, Message)>,
    pub send_message_all_cb: Callback<Message>,
}

pub struct HostArea {
    pub host_props: Rc<RefCell<HostPorps>>,
    pub send_message_all: Callback<Message>,
}

impl HostArea {
    pub fn send_message_to_all(&self, message: Message) {
        self.send_message_all.emit(message);
    }
}

impl Component for HostArea {
    type Message = Msg;

    type Properties = HostAreaProps;

    fn create(ctx: &yew::Context<Self>) -> Self {
        Self { 
            host_props: ctx.props().host_props.clone(),
            send_message_all: ctx.props().send_message_all_cb.clone(),
         }
    }

    fn update(&mut self, _ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::UpdateValue(content) => {
                let host_area_kind = self.host_props.borrow().host_area_kind;
                match host_area_kind {
                    AreaKind::Editor => {
                        self.host_props.clone().borrow_mut().host_editor_content = content.clone();
                    },
                    AreaKind::TextArea => {
                        self.host_props.clone().borrow_mut().host_area_content.set_content(content.clone());
                    },
                }                

                let message = Message::HostToHost {
                             message: content,
                             area_kind: self.host_props.as_ref().borrow().host_area_kind
                };
                let _ = self.send_message_to_all(message);
                false
            },
            Msg::Tick => {
                true
            },
            Msg::SwitchArea(area_kind) => {
                let message = Message::HostSwitchArea { message: area_kind };
                self.send_message_to_all(message);
                true
            },
            Msg::OpenPaint => {
                let message = Message::OpenPaint;
                self.send_message_to_all(message);
                true
            },
            Msg::OnCummunication => {
                let is_communication = self.host_props.borrow_mut().switch_communication();
                let message = Message::OnCummunication { message: *is_communication.borrow() };
                self.send_message_to_all(message);
                true
            }
        }
    }

    fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {

        let text_model = TextModel::create(&ctx.props().host_props.borrow().host_editor_content, Some("java"), None).unwrap();
        // let on_host_editor_cb = &ctx.props().on_host_editor_cb.clone();
        let on_host_editor_cb = ctx.link().callback(|content: String| Msg::UpdateValue(content));

        let render = || {
            match &ctx.props().host_props.clone().borrow().host_area_kind {
                AreaKind::Editor => {
                    html! {
                        <div class="document">
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
                        <div class="document">
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
            let host_props = ctx.props().host_props.clone();
            let send_message_all = self.send_message_all.clone();
            let paint_click = ctx.link().callback(move |_: MouseEvent| {
                let area_kind = &host_props.borrow().host_area_kind;
                match area_kind {
                    AreaKind::Editor => {           
                        let _ = draw::paint::start(&host_props.borrow().host_editor_content, send_message_all.clone());
                    },
                    AreaKind::TextArea => {
                        let _ = draw::paint::start(&host_props.borrow().host_area_content.content, send_message_all.clone());
                    },
                }
                
                Msg::OpenPaint
            });
            let on_communication = ctx.link().callback(|_: MouseEvent| {
                Msg::OnCummunication
            });
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
                            if *(self.host_props.borrow().is_communication.borrow()) {
                                html! { <Icon icon_id={IconId::BootstrapPeopleFill}/> }
                            } else {
                                html! { <Icon icon_id={IconId::BootstrapPeople}/> }
                            }
                        }
                    </button>
                </>
            }
        };
       
        html! {
            <>
                <div class="host-box">
                    { render_batton_bar() }
                    <div class="host-content-box">
                        { render() }
                        <div id="host-paint" class="host-paint">

                        </div>
                    </div>
                    
                </div>
                
            </>
        }
    }
}