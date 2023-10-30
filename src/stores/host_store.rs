use std::{cell::RefCell, rc::Rc, collections::HashMap};

use wasm_peers::{SessionId, UserId, one_to_many::MiniServer};
use yewdux::{store::{Reducer, Store}, prelude::Dispatch};

use crate::{components::multi::host::host_manager::HostManager, models::{client::ClientItem, commons::{AreaKind, InitUser}, video::Video, audio::Audio}, stores::host_store, utils::{inputs::Message, dom::{create_video_id, on_visible_el}}};

use super::{client_items_store::{ClientItemsStore, ClientItemMsg}, client_props_store::{ClientPropsStore, HostClientMsg}, host_props_store::{HostPropsStore, HostHostMsg}, media_store::{MediaStore, HostMediaMsg}};

#[derive(Clone, PartialEq, Store)]
pub struct HostStore {
    pub session_id: Option<SessionId>,
    pub host_manager: Option<Rc<RefCell<HostManager>>>,
}

impl Default for HostStore {
    fn default() -> Self {
        Self { 
            session_id: None,
            host_manager: None,
        }
    }
}

impl HostStore {
    pub fn init(&mut self, session_id: SessionId) {
        let mut host_manager = HostManager::new(session_id);
        let dispatch = Dispatch::<HostStore>::new();
        let on_action = move |msg: host_store::Msg| {
            dispatch.apply(msg);
        };
        host_manager.init(on_action);
        self.session_id = Some(session_id);
        self.host_manager = Some(Rc::new(RefCell::new(host_manager)));
    }

    pub fn get_host_manager(&self) -> Option<Rc<RefCell<HostManager>>> {
        self.host_manager.clone()
    }

    pub fn get_mini_server(&self) -> MiniServer {
        self.host_manager
            .as_ref()
            .expect("cannot get host manager")
            .borrow()
            .mini_server
            .clone()
    }

    pub fn get_players(&self) -> Rc<RefCell<HashMap<UserId, ClientItem>>> {
        Rc::clone(&self.host_manager
            .as_ref()
            .expect("cannot get host manager")
            .borrow()
            .players)
    }

    pub fn get_decoders(&self) -> Rc<RefCell<HashMap<UserId, Rc<RefCell<Video>>>>> {
        self.host_manager
            .as_ref()
            .expect("cannot get host manager")
            .borrow()
            .video_decoders
            .clone()
    }

    pub fn get_audio_decoders(&self) -> Rc<RefCell<HashMap<UserId, Rc<RefCell<Audio>>>>> {
        self.host_manager
            .as_ref()
            .expect("cannot get host manager")
            .borrow()
            .audio_decoders
            .clone()
    }

    pub fn send_message_to_all(&self, mesage: Message) {
        let _ = self.get_mini_server().send_message_to_all(&mesage);
    }
}

pub enum Msg {
    Init(SessionId),
    SendMessage(Message),
    SendMessageToUser(UserId, Message),
    // Host manager actions
    AddClient(UserId),
    DisconnectClient(UserId),
    InitClient(UserId, InitUser),
    ClientSwitchVideo(UserId, bool),
    ClientToClient(UserId, String, AreaKind),
    ClientSwitchArea(UserId, AreaKind),
    // Host manager actions
}

impl Reducer<HostStore> for Msg {
    fn apply(self, mut store: Rc<HostStore>) -> Rc<HostStore> {
        let state = Rc::make_mut(&mut store);
        let client_items_dispatch = Dispatch::<ClientItemsStore>::new();
        let client_area_dispatch = Dispatch::<ClientPropsStore>::new();
        let host_area_dispatch = Dispatch::<HostPropsStore>::new();
        let media_dispatch = Dispatch::<MediaStore>::new();
        match self {
            Msg::Init(session_id) => {
                state.init(session_id);
                let hm = state.get_host_manager();
                media_dispatch.apply(HostMediaMsg::Init(hm));
            }
            Msg::SendMessage(message) => {
                let _ = state.get_mini_server().send_message_to_all(&message);
            }
            Msg::SendMessageToUser(user_id, message) => {
                let _ = state.get_mini_server().send_message(user_id, &message);
            }
            Msg::AddClient(user_id) => {
                host_area_dispatch.apply(HostHostMsg::AddClient(user_id));
                client_items_dispatch.apply(ClientItemMsg::AddClient(user_id));
                media_dispatch.apply(HostMediaMsg::IsScreen(user_id));
            }
            Msg::InitClient(user_id, init_user) => {
                client_items_dispatch.apply(ClientItemMsg::InitClient(user_id, init_user));
            }
            Msg::DisconnectClient(user_id) => {
                client_items_dispatch.apply(ClientItemMsg::DisconnectClient(user_id));
            }
            Msg::ClientSwitchVideo(user_id, message) => {
                let video_id = create_video_id(user_id.to_string());
                let client_logo_id = create_video_id(format!("{}_{}", "client-video-logo", user_id.to_string()));
                on_visible_el(message, &video_id, &client_logo_id);
            }
            Msg::ClientToClient(
                user_id,
                message,
                area_kind
            ) => {
                match area_kind {
                    AreaKind::Editor => {
                        client_items_dispatch.apply(ClientItemMsg::SetEditorContent(user_id, message.clone()));
                        client_area_dispatch.apply(HostClientMsg::SetEditorContent(user_id, message));
                    },
                    AreaKind::TextArea => {
                        client_items_dispatch.apply(ClientItemMsg::SetTextAreaContent(user_id, message.clone()));
                        client_area_dispatch.apply(HostClientMsg::SetTextAreaContent(user_id, message));
                    },
                }
            }
            Msg::ClientSwitchArea(user_id, area_kind) => {
                client_area_dispatch.apply(HostClientMsg::ClientSwitchArea(user_id, area_kind));
                client_items_dispatch.apply(ClientItemMsg::ClientSwitchArea(user_id, area_kind));                
            }
        };

        store
    }

}