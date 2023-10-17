use std::rc::Rc;

use wasm_bindgen::JsCast;
use web_sys::{InputEvent, HtmlTextAreaElement};
use yewdux::{store::{Store, Reducer}, prelude::Dispatch};

use crate::{models::{client::ClientProps, commons::AreaKind}, stores::client_store::{ClientStore, ClientMsg}, utils::inputs::ClientMessage};


#[derive(Clone, PartialEq, Store)]
pub struct ClientPropsStore {
    client_props: Option<ClientProps>,
}

impl Default for ClientPropsStore {
    fn default() -> Self {
        Self { 
            client_props: Some(ClientProps::new()),
        }
    }
}

impl ClientPropsStore {
    pub fn get_client_props(&self) -> &ClientProps {
        self.client_props.as_ref().unwrap()
    }
    pub fn get_mut_client_props(&mut self) -> &mut ClientProps {
        self.client_props.as_mut().unwrap()
    }
}

pub enum ClientPropsMsg {
    SwitchArea(AreaKind),
    UpdateClientValue(String),
    UpdateClientTextArea(InputEvent),
    HostToClient {
        message: String,
        area_kind: AreaKind,
    },
}


impl Reducer<ClientPropsStore> for ClientPropsMsg {
    fn apply(self, mut store: Rc<ClientPropsStore>) -> Rc<ClientPropsStore> {
        let state = Rc::make_mut(&mut store);
        let global_dispatch = Dispatch::<ClientStore>::new();
        match self {
            ClientPropsMsg::SwitchArea(area_kind) => {
                state.get_mut_client_props().set_area_kind(area_kind);
                let message = ClientMessage::ClientSwitchArea { message: area_kind };
                global_dispatch.apply(ClientMsg::SendMessage(message));
            },
            ClientPropsMsg::UpdateClientValue(content) => {
                state.get_mut_client_props().set_editor_content(content.clone());
                state.get_mut_client_props().set_is_write(false);

                let message = ClientMessage::ClientToClient {
                             message: content,
                             area_kind: state.get_client_props().client_area_kind
                };
                global_dispatch.apply(ClientMsg::SendMessage(message)); 
            },
            ClientPropsMsg::UpdateClientTextArea(event) => {
                 let content = event
                    .target()
                    .unwrap()
                    .unchecked_into::<HtmlTextAreaElement>()
                    .value();
                let message = ClientMessage::ClientToClient {
                             message: content,
                             area_kind: state.get_client_props().client_area_kind
                };
                global_dispatch.apply(ClientMsg::SendMessage(message));
            },
            ClientPropsMsg::HostToClient {
                message,
                area_kind
            } => {
                match area_kind {
                    AreaKind::Editor => {
                        state.get_mut_client_props().set_editor_content(message);
                        state.get_mut_client_props().set_is_write(true);
                    },
                    AreaKind::TextArea => {
                        state.get_mut_client_props().set_text_area_content(message);
                    },
                }
            },
        }
        
        store
    }
}