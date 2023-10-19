use std::{rc::Rc, collections::HashMap};

use wasm_bindgen::JsCast;
use wasm_peers::UserId;
use web_sys::{HtmlCanvasElement, HtmlTextAreaElement, InputEvent};
use yew::Callback;
use yewdux::{store::{Store, Reducer}, prelude::Dispatch};

use crate::{models::{host::HostPorps, commons::{AreaKind, InitUser}}, components::multi::draw::paint, utils::{inputs::{PaintAction, Message}, dom::remove_element}, stores::host_store::{self, HostStore}};


#[derive(Clone, PartialEq, Store)]
pub struct HostPropsStore {
    host_props: Option<HostPorps>,
    paints: Option<HashMap<i8, Rc<HtmlCanvasElement>>>,
    paints_f: Option<HashMap<i8, String>>
}

impl Default for HostPropsStore {
    fn default() -> Self {
        Self { 
            host_props: Some(HostPorps::new()),
            paints: Some(HashMap::new()),
            paints_f: Some(HashMap::new()),
        }
    }
}

impl HostPropsStore {
    pub fn get_host_props(&self) -> &HostPorps {
        self.host_props.as_ref().unwrap()
    }
    pub fn get_mut_host_props(&mut self) -> &mut HostPorps {
        self.host_props.as_mut().unwrap()
    }

    pub fn get_paints(&mut self) -> &mut HashMap<i8, Rc<HtmlCanvasElement>> {
        self.paints.as_mut().unwrap()
    }

    pub fn get_mut_paints_f(&mut self) -> &mut HashMap<i8, String> {
        self.paints_f.as_mut().unwrap()
    }

    pub fn get_paints_f(&self) -> & HashMap<i8, String> {
        self.paints_f.as_ref().unwrap()
    }
}

pub enum HostHostMsg {
    AddClient(UserId),
    HostUpdateValue(String),
    HostTextAreaInput(InputEvent),
    SwitchHostArea(AreaKind),
    OpenPaint,
    ClosePaint,
    OnCummunication,
}

impl Reducer<HostPropsStore> for HostHostMsg {
    fn apply(self, mut store: Rc<HostPropsStore>) -> Rc<HostPropsStore> {
        let state = Rc::make_mut(&mut store);
        let dispatch = Dispatch::<HostPropsStore>::new();
        let global_dispatch = Dispatch::<HostStore>::new();
        match self {
            HostHostMsg::AddClient(user_id) => {
                let editor_content = state.get_host_props().host_editor_content.clone();
                let text_area_content = state.get_host_props().host_area_content.content.clone();
                let area_kind = state.get_host_props().host_area_kind;
                let is_communication = state.get_host_props().is_communication;
                let init_user = InitUser {
                    editor_content,
                    text_area_content,
                    area_kind: area_kind.clone(),
                    is_communication
                };
                let message = Message::Init { 
                    message: init_user,      
                };
                global_dispatch.apply(host_store::Msg::SendMessageToUser(user_id, message));
            }
            HostHostMsg::HostUpdateValue(content) => {
                let host_area_kind = state.get_host_props().host_area_kind;
                match host_area_kind {
                    AreaKind::Editor => {
                        state.get_mut_host_props().host_editor_content = content.clone();
                    },
                    AreaKind::TextArea => {
                        state.get_mut_host_props().host_area_content.set_content(content.clone());
                    },
                }
                let message = Message::HostToHost {
                             message: content,
                             area_kind: state.get_host_props().host_area_kind
                };
                global_dispatch.apply(host_store::Msg::SendMessage(message));
            }
            HostHostMsg::HostTextAreaInput(event) => {
                let content = event
                    .target()
                    .unwrap()
                    .unchecked_into::<HtmlTextAreaElement>()
                    .value();
                dispatch.apply(HostHostMsg::HostUpdateValue(content));
            }
            HostHostMsg::SwitchHostArea(area_kind) => {
                state.get_mut_host_props().set_host_area_kind(area_kind);

                let message = Message::HostSwitchArea { message: area_kind };
                global_dispatch.apply(host_store::Msg::SendMessage(message));
            }
            HostHostMsg::OpenPaint => {
                let area_kind = state.get_host_props().host_area_kind;
                match area_kind {
                    AreaKind::Editor => {
                        let content = state.get_host_props().host_editor_content.clone();
                        state.get_mut_paints_f().insert(1, content);
                    },
                    AreaKind::TextArea => {
                        let content = state.get_host_props().host_area_content.content.clone();
                        state.get_mut_paints_f().insert(1, content);
                    },
                }
                let message = Message::OpenPaint;
                global_dispatch.apply(host_store::Msg::SendMessage(message));
            }
            HostHostMsg::ClosePaint => {
                remove_element("draw-canvas".to_string());
                let message = Message::ClosePaint;
                global_dispatch.apply(host_store::Msg::SendMessage(message));
            }
            HostHostMsg::OnCummunication => {
                let is_communication = state.get_mut_host_props().switch_communication();
                let message = Message::OnCummunication { message: is_communication };
                global_dispatch.apply(host_store::Msg::SendMessage(message));
            }     
        }

        store

    }
}

pub enum HostPropsMsg {
    InitHost(InitUser),
    HostSwitchArea(AreaKind),
    HostToHost {
        message: String,
        area_kind: AreaKind,
    },
    OpenPaint,
    HostPaint {
        offset_x: f64,
        offset_y: f64,
        action: PaintAction,
    },
}


impl Reducer<HostPropsStore> for HostPropsMsg {
    fn apply(self, mut store: Rc<HostPropsStore>) -> Rc<HostPropsStore> {
        let state = Rc::make_mut(&mut store);
        match self {
            HostPropsMsg::HostToHost { 
                message,
                area_kind
            } => {
                match area_kind {
                    AreaKind::Editor => {
                        state.get_mut_host_props().set_editor_content(message);
                    },
                    AreaKind::TextArea => {
                        state.get_mut_host_props().host_area_content.set_content(message)
                    },
                }
            }
            HostPropsMsg::InitHost(user) => {
                state.get_mut_host_props().host_area_content.set_content(user.text_area_content);
                state.get_mut_host_props().set_editor_content(user.editor_content);
                state.get_mut_host_props().set_host_area_kind(user.area_kind);
                state.get_mut_host_props().set_communication(user.is_communication);
            },
            HostPropsMsg::HostSwitchArea(area_kind) => {
                state.get_mut_host_props().set_host_area_kind(area_kind);
            },
            HostPropsMsg::OpenPaint => {
                match state.get_host_props().host_area_kind {
                    AreaKind::Editor => {
                        let canvas = paint::start(&state.get_host_props().host_editor_content, Callback::default(), false)
                            .expect("cannot get canvas");
                        state.get_paints().insert(1, canvas);
                    },
                    AreaKind::TextArea => {
                        let canvas = paint::start(&state.get_host_props().host_area_content.content, Callback::default(), false)
                            .expect("cannot get canvas");
                        state.get_paints().insert(2, canvas);
                    },
                }
            }
            HostPropsMsg::HostPaint { 
                offset_x,
                offset_y,
                action
            } => {
                let key = 1;
                let canvas = state.get_paints().get(&key).expect("cannot get canvas");
                let context = canvas
                    .get_context("2d")
                    .expect("cannot get canvas")
                    .unwrap()
                    .dyn_into::<web_sys::CanvasRenderingContext2d>()
                    .expect("cannot get canvas");
                match action {
                    PaintAction::Down => {
                        context.begin_path();
                        context.move_to(offset_x, offset_y);
                    },
                    PaintAction::Move => {
                        context.line_to(offset_x, offset_y);
                        context.stroke();
                        context.begin_path();
                        context.move_to(offset_x, offset_y);
                    },
                    PaintAction::Up => {
                        context.line_to(offset_x, offset_y);
                        context.stroke();
                    },
                };
            }
        }
        store
    }
}