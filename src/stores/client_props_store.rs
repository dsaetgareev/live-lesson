use std::rc::Rc;

use wasm_bindgen::JsCast;
use wasm_peers::UserId;
use web_sys::{InputEvent, HtmlTextAreaElement};
use yewdux::{store::{Store, Reducer}, prelude::Dispatch};

use crate::{models::{client::{ClientProps, ClientItem}, commons::{AreaKind, InitUser}}, stores::{client_store::{ClientStore, ClientMsg}, host_store::{HostStore, self}, client_items_store::{ClientItemsStore, ClientItemMsg}}, utils::inputs::{ClientMessage, Message}};


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
    SendStateToHost,
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
            ClientPropsMsg::SendStateToHost => {
                let editor_content = state.get_client_props().client_editor_content.clone();
                let text_area_content = state.get_client_props().client_text_area.content.clone();
                let area_kind = state.get_client_props().client_area_kind;
                let init_user = InitUser {
                    editor_content,
                    text_area_content,
                    area_kind: area_kind.clone(),
                    is_communication: false
                };
                let message = ClientMessage::InitClient { message: init_user };
                global_dispatch.apply(ClientMsg::SendMessage(message));
            }
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
                state.get_mut_client_props().set_text_area_content(content.clone());
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

pub enum HostClientMsg {
    SetTextAreaContent(UserId, String),
    SetEditorContent(UserId, String),
    ClientSwitchArea(UserId, AreaKind),
    HostClientToClient(String),
    SetFromChoosedItem(String, ClientItem),
}

impl Reducer<ClientPropsStore> for HostClientMsg {
    fn apply(self, mut store: Rc<ClientPropsStore>) -> Rc<ClientPropsStore> {
        let state = Rc::make_mut(&mut store);
        let global_dispatch = Dispatch::<HostStore>::new();
        let client_item_dispatch = Dispatch::<ClientItemsStore>::new();
        match self {
            HostClientMsg::SetTextAreaContent(
                user_id,
                message,
            ) => {
                if state.get_client_props().client_id == user_id.to_string() {
                    state.get_mut_client_props().set_text_area_content(message);
                }
            }
            HostClientMsg::SetEditorContent(user_id, content) => {
                if state.get_client_props().client_id == user_id.to_string() {
                    state.get_mut_client_props().set_editor_content(content);
                    state.get_mut_client_props().is_write = true;
                }
            },
            HostClientMsg::ClientSwitchArea(user_id, area_kind ) => {
                if state.get_client_props().client_id == user_id.to_string() {
                    state.get_mut_client_props().set_area_kind(area_kind);
                }
            }
            HostClientMsg::HostClientToClient(content) => {
                let client_id = state.get_client_props().client_id.clone();
                if !client_id.is_empty() {
                    let user_id: UserId = UserId::new(client_id.parse::<u64>().unwrap());
                    let area_kind = state.get_client_props().client_area_kind;
                    match area_kind {
                        AreaKind::Editor => {
                            client_item_dispatch.apply(ClientItemMsg::SetEditorContent(user_id, content.clone()));
                            state.get_mut_client_props().set_editor_content(content.clone());
                        },
                        AreaKind::TextArea => {
                            client_item_dispatch.apply(ClientItemMsg::SetTextAreaContent(user_id, content.clone()));
                            state.get_mut_client_props().set_text_area_content(content.clone());
                        },
                    }
                                                            
                    let message = Message::HostToClient {
                        message: content,
                        area_kind,
                    };
                    global_dispatch.apply(host_store::Msg::SendMessageToUser(user_id, message));
                    state.get_mut_client_props().set_is_write(false);
                }
            }
            HostClientMsg::SetFromChoosedItem(client_id, client_item) => {
                state.get_mut_client_props().set_client_id(client_id.clone());
                state.get_mut_client_props().set_area_kind(client_item.area_kind);
                state.get_mut_client_props().set_editor_content(client_item.editor_content);
                state.get_mut_client_props().set_text_area_content(client_item.text_area_content);
                state.get_mut_client_props().is_write = true;
            }
        }
        store
    }
}