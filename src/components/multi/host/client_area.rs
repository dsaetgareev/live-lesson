use std::{cell::RefCell, rc::Rc, collections::HashMap};

use monaco::api::TextModel;
use wasm_bindgen::JsCast;
use wasm_peers::UserId;
use web_sys::{InputEvent, HtmlTextAreaElement};
use yew::prelude::*;
use yewdux::prelude::*;

use crate::{models::{client::{ClientProps, ClientItem}, commons::AreaKind}, components::editor::editor::EditorWrapper, utils::inputs::Message, stores::host_store::{HostStore, self}};

const TEXTAREA_ID_CLIENT: &str = "client-textarea";

pub enum Msg {
    SendClient(String),
    Tick,
}

// #[derive(PartialEq, Properties)]
// pub struct ClientAreaProps {
//     pub client_props: Rc<RefCell<ClientProps>>,
//     pub players: Rc<RefCell<HashMap<UserId, ClientItem>>>,
//     pub send_message_cb: Callback<(UserId, Message)>,
//     pub on_tick: Callback<String>,
// }

#[function_component(ClientArea)]
pub fn client_area() -> Html {

    let (state, dispatch) = use_store::<HostStore>();     

    let on_host_editor_cb = {
        let dispatch = dispatch.clone();
        Callback::from(move |content: String| dispatch.apply(host_store::Msg::HostClientToClient(content)))
    };

    let render = || {
        let area_kind = state.client_props
            .as_ref()
            .expect("not initialized client props")
            .borrow()
            .client_area_kind;
        match area_kind {
            AreaKind::Editor => {
                let text_model = TextModel::create(&state.client_props.as_ref().unwrap().borrow().client_editor_content, Some("java"), None).unwrap();
       
                let is_write = state.client_props.as_ref().unwrap().borrow().is_write;
                html! {
                    <div class="document">
                        <EditorWrapper on_cb={ on_host_editor_cb.clone() } text_model={ text_model.clone() } is_write={ is_write }/>
                    </div>
                }
            },
            AreaKind::TextArea => {
                let on_host_editor_cb = on_host_editor_cb.clone();
                let oninput = Callback::from(move |e: InputEvent| {
                    let content = e
                        .target()
                        .unwrap()
                        .unchecked_into::<HtmlTextAreaElement>()
                        .value();
                    on_host_editor_cb.emit(content);
                });
                let value = state.client_props.as_ref().unwrap().borrow().client_text_area.content.clone();

                html! {
                    <div class="col document">
                        <textarea id={ TEXTAREA_ID_CLIENT } value={ value } { oninput } class="document" cols="100" rows="30" />
                    </div>
                }
            },
        }
    };

    html! {
        <>
            {render()}
        </>
    }
}

// pub struct ClientArea {
//     pub client_props: Rc<RefCell<ClientProps>>,
//     pub send_message_cb: Callback<(UserId, Message)>,
//     pub players: Rc<RefCell<HashMap<UserId, ClientItem>>>,
// }

// impl ClientArea {

//     pub fn send_message(&self, message: Message) {
//         let user_id: UserId = UserId::new(self.client_props.borrow().client_id.parse::<u64>().unwrap());
//         self.send_message_cb.emit((user_id, message));
//     }
// }

// impl Component for ClientArea {
//     type Message = Msg;

//     type Properties = ClientAreaProps;

//     fn create(ctx: &yew::Context<Self>) -> Self {
//         Self { 
//             client_props: ctx.props().client_props.clone(),
//             send_message_cb: ctx.props().send_message_cb.clone(),
//             players: ctx.props().players.clone(),
//          }
//     }

//     fn update(&mut self, ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
//         match msg {
//             Msg::SendClient(content) => {
//                 if self.client_props.borrow().client_id.is_empty() {
//                     false
//                 } else {
//                     let user_id: UserId = UserId::new(self.client_props.borrow().client_id.parse::<u64>().unwrap());
//                     let area_kind = self.client_props.borrow().client_area_kind;
//                     // let client_item = self.players.borrow_mut().get(&user_id);
//                     match self.players.borrow_mut().get_mut(&user_id) {
//                         Some(client_item) => {
//                             match area_kind {
//                                 AreaKind::Editor => {
//                                     client_item.set_editor_content(content.clone());
//                                     self.client_props.borrow_mut().set_editor_content(content.clone());
//                                 },
//                                 AreaKind::TextArea => {
//                                     client_item.set_text_area_content(content.clone());
//                                     self.client_props.borrow_mut().set_text_area_content(content.clone());
//                                 },
//                             }
//                         },
//                         None => {
//                             log::error!("cannot find client item, id: {}", user_id.to_string());
//                         },
//                     }
                                        
//                     let message = Message::HostToClient {
//                         message: content,
//                         area_kind,
//                     };
//                     let _ = self.send_message(message);
//                     self.client_props.borrow_mut().is_write = false;
//                     ctx.props().on_tick.emit("value".to_string());
//                     true
//                 }
                
//             },
//             Msg::Tick => {
//                 true
//             }
//         }
//     }

//     fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
//         let text_model = TextModel::create(&ctx.props().client_props.borrow().client_editor_content, Some("java"), None).unwrap();
//         // let on_host_editor_cb = &ctx.props().on_host_editor_cb.clone();
//         let on_host_editor_cb = ctx.link().callback(|content: String| Msg::SendClient(content));

//         let render = || {
//             match &ctx.props().client_props.clone().borrow().client_area_kind {
//                 AreaKind::Editor => {
//                     let is_write = &ctx.props().client_props.borrow().is_write;
//                     html! {
//                         <div class="document">
//                             <EditorWrapper on_cb={ on_host_editor_cb.clone() } text_model={ text_model.clone() } is_write={ is_write }/>
//                         </div>
//                     }
//                 },
//                 AreaKind::TextArea => {
//                     let on_host_editor_cb = on_host_editor_cb.clone();
//                     let oninput = ctx.link().callback(move |e: InputEvent| {
//                         let content = e
//                             .target()
//                             .unwrap()
//                             .unchecked_into::<HtmlTextAreaElement>()
//                             .value();
//                         on_host_editor_cb.emit(content);
//                         Msg::Tick
//                     });
//                     let value = ctx.props().client_props.borrow().client_text_area.content.clone();

//                     html! {
//                         <div class="col document">
//                             <textarea id={ TEXTAREA_ID_CLIENT } value={ value } { oninput } class="document" cols="100" rows="30" />
//                         </div>
//                     }
//                 },
//             }
//         };
       
//         html! {
//             <>
//                 { render() }                
//             </>
//         }
//     }
// }