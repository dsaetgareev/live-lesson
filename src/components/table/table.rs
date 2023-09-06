use std::{rc::Rc, cell::RefCell};

use serde::{Serialize, Deserialize};
use wasm_bindgen::JsCast;
use wasm_peers::{get_random_session_id, SessionId, ConnectionType, many_to_many::NetworkManager};
use web_sys::HtmlElement;
use yew::prelude::*;

use crate::utils::{self, dom::get_window, dom::get_table_td};


#[derive(Clone, Serialize, Deserialize)]
pub struct Cell {
    id: String,
    content: String,
}

impl Cell {
    fn new(id: String, content: String) -> Self {
        Self { id, content }
    }
}

pub enum Msg {
    UpdateValue(Cell),
}



pub struct Table {
    network_manager: NetworkManager,
    items: Vec<Vec<Cell>>,
    colunms_headers: Vec<String>,
}

impl Component for Table {
    type Message = Msg;

    type Properties = ();



    fn create(_ctx: &yew::Context<Self>) -> Self {
        let query_params = utils::dom::get_query_params().expect("failed to get query params, aborting");
        let session_id = query_params.get("session_id").map_or_else(
            || {
                let location = get_window().expect("failed to get a window").location();
                let generated_session_id = get_random_session_id();
                query_params.append("session_id", generated_session_id.as_str());
                let search: String = query_params.to_string().into();
                location.set_search(&search).unwrap();
                generated_session_id
            },
            SessionId::new,
        );

        let is_ready = Rc::new(RefCell::new(false));
        let connection_type = ConnectionType::StunAndTurn {
            stun_urls: env!("STUN_SERVER_URLS").to_string(),
            turn_urls: env!("TURN_SERVER_URLS").to_string(),
            username: env!("TURN_SERVER_USERNAME").to_string(),
            credential: env!("TURN_SERVER_CREDENTIAL").to_string(),
        };
        let mut network_manager = NetworkManager::new(
            concat!(env!("SIGNALING_SERVER_URL"), "/many-to-many"),
            session_id.clone(),
            connection_type,
        )
        .unwrap();

        let on_open_callback = {
            let mini_server = network_manager.clone();
            let is_ready = Rc::clone(&is_ready);
            move |user_id| {
                let text_area = match utils::dom::get_table_td("A1") {
                    Ok(text_area) => text_area,
                    Err(err) => {
                        log::error!("failed to get textarea: {:#?}", err);
                        return;
                    }
                };
                if !*is_ready.borrow() {
                    *is_ready.borrow_mut() = true;
                }
                let value = text_area.inner_text();
                log::info!("message from value {}", value.clone());
                if !value.is_empty() {
                    let cell = Cell::new("A1".to_string(), value);
                    let message = serde_json::to_string::<Cell>(&cell).unwrap();
                    mini_server
                        .send_message(user_id, &message)
                        .expect("failed to send current input to new connection");
                }
            }
        };

        let on_message_callback = {
            move |_, message: String| {
                let cell = serde_json::from_str::<Cell>(&message).unwrap();
                let element = get_table_td(&cell.id).unwrap();
                element.set_inner_text(&cell.content);
            }
        };

        network_manager.start(on_open_callback, on_message_callback);

        let (colunms_headers, items) = create_table_items();
        Self {
            network_manager,
            items,
            colunms_headers,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Self::Message::UpdateValue(cell) => {
                log::info!("update");
                log::info!("{}", cell.id);
                let message = serde_json::to_string::<Cell>(&cell).unwrap();
                self.network_manager.send_message_to_all(&message);
                true
            },
        }
    }

    fn view(&self, _ctx: &yew::Context<Self>) -> yew::Html {

        let oninput = _ctx.link().callback(|input: InputEvent| {
            let target: HtmlElement = input
            .target()
            .unwrap()
            .dyn_into()
            .unwrap();
            log::info!("{}", target.inner_text());
            Self::Message::UpdateValue(Cell::new(target.id(), target.inner_text()))
        });

        let render_item = |cell: &Cell| {
            let id = &cell.id;
            let id = id.to_string();
            html! {
                <>
                     <td id={ id } contenteditable="true" { oninput }>
                            { &cell.content }
                     </td>
                </>
            }
        };

        let render_row = |row: &Vec<Cell>| {
            html! {
                <>
                    <tr>
                        { for row
                            .iter()
                            .map(|cell| {
                                    render_item.clone()(&cell)
                                }
                            )
                        }
                    </tr>
                </>
            }
            
        };

        let render_header = |header: &str| {
            html! {
                <>
                    <th>
                        { header }
                    </th>
                </>
            }
        };

        html!(
            <div class="main">
                <div class="card">
                    <header>
                        {"Items test: "}
                    </header>
                    <div class="card-body scroll">
                        <table class="listview">
                            <tr>
                                { for self.colunms_headers.iter().map(|head|render_header(&head)) }
                            </tr>
                            { for self.items.iter().map(|row| render_row(row)) }
                        </table>
                    </div>
                    <footer>
                        {"footer"}
                    </footer>
                </div>
            </div>
        )
    }

}

fn create_table_items() -> (Vec<String>, Vec<Vec<Cell>>) {
    let colunms_headers: Vec<char> = " ABCDEFGHIJKLMNOPQRSTUVWXYZ".chars().collect();
    let colunms_headers: Vec<String> = colunms_headers.into_iter().map(|item| item.to_string()).collect();
    let mut items: Vec<Vec<Cell>> = Vec::new();
    for n in 1..=100 {
        let row: Vec<Cell> = colunms_headers.iter().map(|item| {
            let id = item.to_string() + n.to_string().as_str();
            return Cell::new(id.clone(), "".to_string())
        }).collect();
        items.push(row);
    }

    (colunms_headers, items)
}
