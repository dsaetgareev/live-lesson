use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlInputElement, HtmlTextAreaElement, UrlSearchParams, Window, HtmlElement};
use yew::NodeRef;

pub fn global_window() -> Window {
    web_sys::window().expect("there was no window global object!")
}

pub fn get_window() -> crate::Result<Window> {
    web_sys::window().ok_or_else(|| crate::Error::MissingElement("window node".to_owned()))
}

pub fn get_query_params() -> crate::Result<UrlSearchParams> {
    let search = get_window()?.location().search().unwrap();
    UrlSearchParams::new_with_str(&search)
        .map_err(|err| crate::Error::FailedToCreateUrlSearchParams(format!("{:?}", err)))
}

pub fn get_query_params_multi() -> UrlSearchParams {
    let search = global_window().location().search().unwrap();
    UrlSearchParams::new_with_str(&search).unwrap()
}

fn get_element(id: &str) -> crate::Result<Element> {
    get_window()?
        .document()
        .ok_or_else(|| crate::Error::MissingElement("document node".to_owned()))?
        .get_element_by_id(id)
        .ok_or_else(|| crate::Error::MissingElement(format!("element with id '{}'", id)))
}

pub fn get_text_area(id: &str) -> crate::Result<HtmlTextAreaElement> {
    get_element(id)?
        .dyn_into::<HtmlTextAreaElement>()
        .map_err(|err| {
            crate::Error::UnexpectedElement(format!("element is not an textarea: {:?}", err))
        })
}

pub fn get_table_td(id: &str) -> crate::Result<HtmlElement> {
    get_element(id)?
        .dyn_into::<HtmlElement>()
        .map_err(|err| {
            crate::Error::UnexpectedElement(format!("element is not an HtmlElement: {:?}", err))
        })
}

pub fn get_input(id: &str) -> crate::Result<HtmlInputElement> {
    get_element(id)?
        .dyn_into::<HtmlInputElement>()
        .map_err(|err| {
            crate::Error::UnexpectedElement(format!("element is not an input: {:?}", err))
        })
}

pub fn get_text_area_from_noderef(node_ref: &NodeRef) -> Result<HtmlTextAreaElement, crate::Error> {
    let text_area = node_ref
        .cast::<HtmlTextAreaElement>()
        .ok_or_else(|| crate::Error::MissingElement("not element node ref".to_owned()))
        .expect("msg")
        .dyn_into::<HtmlTextAreaElement>()
        .map_err(|err| {
            crate::Error::UnexpectedElement(format!("not element node ref: {:?}", err))
        });
    text_area
}
