use std::{rc::Rc, collections::HashMap};

use wasm_bindgen::JsCast;
use web_sys::HtmlCanvasElement;
use yew::Callback;
use yewdux::store::{Store, Reducer};

use crate::{models::{host::HostPorps, commons::{AreaKind, InitUser}}, components::multi::draw::paint, utils::inputs::PaintAction};


#[derive(Clone, PartialEq, Store)]
pub struct HostPropsStore {
    host_props: Option<HostPorps>,
    paints: Option<HashMap<i8, Rc<HtmlCanvasElement>>>
}

impl Default for HostPropsStore {
    fn default() -> Self {
        Self { 
            host_props: Some(HostPorps::new()),
            paints: Some(HashMap::new()),
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
                        let canvas = paint::start(&state.get_host_props().host_editor_content, Callback::default())
                            .expect("cannot get canvas");
                        state.get_paints().insert(1, canvas);
                    },
                    AreaKind::TextArea => {
                        let canvas = paint::start(&state.get_host_props().host_area_content.content, Callback::default())
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