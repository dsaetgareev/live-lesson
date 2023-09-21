use yew::{html, Component, Context, Html};
use yew_router::prelude::*;

use crate::components::document::document::Document;
use crate::components::home::Home;
use crate::components::multi::multi::Multi;
use crate::components::table::table::Table;

#[derive(Clone, Routable, PartialEq, Eq)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/document")]
    Document,
    #[at("/table")]
    Table,
    #[at("/multi")]
    Multi,
}

pub struct App;

impl Component for App {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        true
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <BrowserRouter>
                <main>
                    <Switch<Route> render={switch} />
                </main>
            </BrowserRouter>
        }
    }
}

fn switch(routes: Route) -> Html {
    #[allow(clippy::let_unit_value)] // html! macro messes something up
    match routes.clone() {
        Route::Home => {
            html! { <Home /> }
        }
        Route::Document => {
            html! { <Document /> }
        }
        Route::Table => {
            html! { <Table /> }
        }
        Route::Multi => {
            html! { <Multi /> }
        }
    }
}
