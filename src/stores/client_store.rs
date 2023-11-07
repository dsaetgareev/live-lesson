use std::{rc::Rc, cell::RefCell};
use wasm_peers::{SessionId, one_to_many::MiniClient, many_to_many::NetworkManager};
use yewdux::{store::{Store, Reducer}, prelude::Dispatch};

use crate::{components::multi::client::client_manager::ClientManager, models::{audio::Audio, commons::{AreaKind, InitUser}}, utils::{inputs::{ClientMessage, ManyMassage, PaintAction}, dom::{on_visible_el, switch_visible_el}}};

use super::{client_props_store::{ClientPropsStore, ClientPropsMsg}, host_props_store::{HostPropsStore, ClientHostPropsMsg}, media_store::{MediaStore, ClientMediaMsg}};

#[derive(Clone, PartialEq, Store)]
pub struct ClientStore {
    session_id: Option<SessionId>,
    client_manager: Option<Rc<RefCell<ClientManager>>>,
    audio: Option<Audio>,
}

impl Default for ClientStore {
    fn default() -> Self {
        Self { 
            session_id: Default::default(),
            client_manager: Default::default(),
            audio: Default::default(),
        }
    }
}

impl ClientStore {
    pub fn init(&mut self, session_id: SessionId) {
        self.session_id = Some(session_id);
        let client_manager = ClientManager::new(session_id);
        self.client_manager = Some(Rc::new(RefCell::new(client_manager)));
    }

    pub fn get_client_manager(&self) -> Option<Rc<RefCell<ClientManager>>> {
        self.client_manager.clone()
    }

     pub fn get_mini_client(&self) -> MiniClient {
        self.client_manager
            .as_ref()
            .expect("cannot get the client manager")
            .borrow()
            .mini_client
            .clone()
    }

    pub fn get_many_network_manager(&self) -> NetworkManager {
        self.client_manager
            .as_ref()
            .expect("cannot get the networr manager")
            .borrow()
            .network_manager
            .clone()
    }
}

pub enum ClientMsg {
    Init(SessionId),
    InitClientManager,
    SendStateToHost,
    SendMessage(ClientMessage),
    SendManyMessage(ManyMassage),
    // Client manager action
    HostToHost {
        message: String,
        area_kind: AreaKind,
    },
    HostToClient {
        message: String,
        area_kind: AreaKind,
    },
    InitHostAra(InitUser),
    HostIsScreenShare(bool),
    HostSwitchArea(AreaKind),
    OpenPaint,
    HostPaint {
        offset_x: f64,
        offset_y: f64,
        action: PaintAction,
    },
    OnCummunication {
        message: bool
    }
    // Client manager action
}

impl Reducer<ClientStore> for ClientMsg {
    fn apply(self, mut store: Rc<ClientStore>) -> Rc<ClientStore> {
        let state = Rc::make_mut(&mut store);
        let dispatch = Dispatch::<ClientStore>::new();
        let client_props_dispatch = Dispatch::<ClientPropsStore>::new();
        let host_props_dispatch = Dispatch::<HostPropsStore>::new();
        let media_dispatch = Dispatch::<MediaStore>::new();
        match self {
            ClientMsg::Init(session_id) => {
                state.init(session_id);
                let cm = state.get_client_manager();
                media_dispatch.apply(ClientMediaMsg::Init(cm));
            }
            ClientMsg::InitClientManager => {
                log::error!("init client manager");
                let dispatch = dispatch.clone();
                let on_action = move |msg: ClientMsg| {
                    dispatch.apply(msg);
                };
                state.get_client_manager().unwrap().borrow_mut().init(on_action);
                state.get_client_manager().unwrap().borrow_mut().many_init();
            }
            ClientMsg::SendStateToHost => {
                client_props_dispatch.apply(ClientPropsMsg::SendStateToHost);
            }
            ClientMsg::SendMessage(message) => {
                let _ = state.get_mini_client().send_message_to_host(&message);
            }
            ClientMsg::SendManyMessage(message) => {
                let _ = state.get_many_network_manager().send_message_to_all(&message);
            }
            // Client manager action
            ClientMsg::HostToHost { 
                message,
                area_kind, 
            } => {
                host_props_dispatch.apply(ClientHostPropsMsg::HostToHost { message, area_kind })
            }
            ClientMsg::HostToClient {
                message,
                area_kind
            } => {
                client_props_dispatch.apply(ClientPropsMsg::HostToClient { message, area_kind })
            },
            ClientMsg::InitHostAra(user) => {
                host_props_dispatch.apply(ClientHostPropsMsg::InitHost(user));
            }
            ClientMsg::HostIsScreenShare(is_share) => {
                on_visible_el(is_share, "container", "shcreen_container");
                switch_visible_el(!is_share, "video-container");         
            }
            ClientMsg::HostSwitchArea(area_kind) => {
                host_props_dispatch.apply(ClientHostPropsMsg::HostSwitchArea(area_kind));
            }
            ClientMsg::OpenPaint => {
                host_props_dispatch.apply(ClientHostPropsMsg::OpenPaint);
            }
            ClientMsg::HostPaint { 
                offset_x,
                offset_y,
                action
            } => {
                host_props_dispatch.apply(ClientHostPropsMsg::HostPaint { offset_x, offset_y, action })
            }
            ClientMsg::OnCummunication { message } => {
                media_dispatch.apply(ClientMediaMsg::OnCummunication(message));
            }
        }
        store
    }
}

