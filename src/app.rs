use yew::{html, Html, function_component};
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

#[function_component(App)]
pub fn app() -> Html {
   
    html! {
        <BrowserRouter>
            <main>
                <Switch<Route> render={switch} />
            </main>
        </BrowserRouter>
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
