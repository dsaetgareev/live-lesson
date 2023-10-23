use std::{collections::HashMap, rc::Rc};

use wasm_bindgen::JsCast;
use wasm_peers::UserId;
use web_sys::{HtmlElement, MouseEvent};
use yewdux::{store::{Store, Reducer}, prelude::Dispatch};

use crate::{models::{client::ClientItem, commons::{AreaKind, InitUser}}, utils::dom::{remove_element, create_video_id}};

use super::client_props_store::{ClientPropsStore, HostClientMsg};


#[derive(Clone, PartialEq, Store)]
pub struct ClientItemsStore {
    players: HashMap<UserId, ClientItem>,
}

impl Default for ClientItemsStore {
    fn default() -> Self {
        Self { 
            players: HashMap::new(),
        }
    }
}

impl ClientItemsStore {
    pub fn get_players(&self) -> HashMap<UserId, ClientItem> {
        self.players.clone()
    }

    pub fn get_mut_players(&mut self) -> &mut HashMap<UserId, ClientItem> {
        &mut self.players
    }
}

pub enum ClientItemMsg {
    AddClient(UserId),
    InitClient(UserId, InitUser),
    SetTextAreaContent(UserId, String),
    SetEditorContent(UserId, String),
    ClientSwitchArea(UserId, AreaKind),
    ChooseItem(MouseEvent),
    SwitchSpeakers(String),
    SwitchVideo(String),
    DisconnectClient(UserId),
}

impl Reducer<ClientItemsStore> for ClientItemMsg {
    fn apply(self, mut store: Rc<ClientItemsStore>) -> Rc<ClientItemsStore> {
        let state = Rc::make_mut(&mut store);
        let client_area_dispatch = Dispatch::<ClientPropsStore>::new();
        match self {
            ClientItemMsg::AddClient(user_id) => {
                state.players
                    .insert(user_id, ClientItem::new(AreaKind::TextArea));
            }
            ClientItemMsg::InitClient(user_id, init_user) => {
                let client_item = state.players.get_mut(&user_id).unwrap();
                client_item.set_area_kind(init_user.area_kind);
                client_item.set_editor_content(init_user.editor_content);
                client_item.set_text_area_content(init_user.text_area_content);
            }
            ClientItemMsg::SetTextAreaContent(user_id, content) => {
                match state.players.get_mut(&user_id) {
                    Some(client_item) => {
                        client_item.set_area_kind(AreaKind::TextArea);
                        client_item.set_text_area_content(content);                      
                    },
                    None => {
                        log::error!("cannot find client item, id: {}", user_id.to_string());
                    },
                }
            },
            ClientItemMsg::SetEditorContent(user_id, content) => {
                match state.players.get_mut(&user_id) {
                    Some(client_item) => {
                        client_item.set_area_kind(AreaKind::Editor);
                        client_item.set_editor_content(content);                      
                    },
                    None => {
                        log::error!("cannot find client item, id: {}", user_id.to_string());
                    },
                }
            },
            ClientItemMsg::ClientSwitchArea(user_id, area_kind) => {
                match state.players.get_mut(&user_id) {
                    Some(client_item) => {
                        client_item.set_area_kind(area_kind)
                    },
                    None => todo!(),
                }
            }
            ClientItemMsg::ChooseItem(event) => {
                let target: HtmlElement = event
                    .target()
                    .unwrap()
                    .dyn_into()
                    .unwrap();
                let client_id = target.get_attribute("client_id").unwrap();
                let client_item = state
                    .players
                    .get(&UserId::new(client_id.parse::<u64>().unwrap()))
                    .unwrap()
                    .clone();
                client_area_dispatch.apply(HostClientMsg::SetFromChoosedItem(client_id, client_item));                
            }
            ClientItemMsg::SwitchSpeakers(_speakers_id) => {
                
            },
            ClientItemMsg::SwitchVideo(_speakers_id) => {
                
            },
            ClientItemMsg::DisconnectClient(user_id) => {
                match state.players.get(&user_id) {
                    Some(_client_item) => {
                        let box_id = format!("item-box-{}", create_video_id(user_id.to_string()));
                        state.players
                            .remove(&user_id)
                            .expect("cannot remove clietn");
                        remove_element(box_id);
                    },
                    None => {
                        log::error!("not found client id: {}", user_id);
                    },
                }
            }
        }
        store

    }

}