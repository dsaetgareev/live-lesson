use std::cell::Cell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::{HtmlCanvasElement, MouseEvent};
use yew::{Callback, Component, Properties, NodeRef, html, use_node_ref, Html, function_component, use_effect, use_state_eq};
use yew_icons::{IconId, Icon};
use yewdux::prelude::use_store;

use crate::models::commons::AreaKind;
use crate::stores::host_props_store::{HostPropsStore, HostHostMsg};
use crate::utils;
use crate::utils::inputs::{Message, PaintAction};

pub enum Msg {

}

#[derive(PartialEq, Properties)]
pub struct CurrentProps {
    #[prop_or_default]
    pub content: String,
    #[prop_or_default]
    pub send_message_all_cb: Callback<Message>,
    pub is_host: bool
}

#[function_component(PaintF)]
pub fn paint(props: &CurrentProps) -> Html {

    let (state, dispatch) = use_store::<HostPropsStore>();

    let content = use_state_eq(|| props.content.clone());

    let canvas = use_node_ref();

    use_effect({
        let canvas = canvas.clone();
        let content = content.clone();
        let send_message_all_cb = props.send_message_all_cb.clone();
        move || {
            let canvas = canvas.cast::<HtmlCanvasElement>();
            match canvas {
                Some(canvas) => {
                    let context = canvas
                        .get_context("2d")
                        .unwrap()
                        .unwrap()
                        .dyn_into::<web_sys::CanvasRenderingContext2d>()
                        .unwrap();
                    canvas.set_width(600);
                    canvas.set_height(500);
                    context.set_font("20px Arial");
                    draw_content(&content, &context);
                    host_action(&canvas, context, send_message_all_cb);
                },
                None => {
                    log::error!("none canvas element");
                },
            }
        }
    });

    let editor_click = {
        let dispatch = dispatch.clone();
        move |_e: MouseEvent| {
            dispatch.apply(HostHostMsg::ClosePaint);
        }
    };

    html! {
        <div>
            <button>
                <Icon icon_id={IconId::FontAwesomeSolidCode} onclick={ editor_click }/>
            </button>
            <canvas id="draw-canvas" ref={ canvas } class="paint"></canvas>
        </div>
    }
}


pub fn start(
    content: &str,
    send_message_all_cb: Callback<Message>,
    is_host: bool,
) -> Result<Rc<HtmlCanvasElement>, JsValue> {
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document
        .create_element("canvas")?
        .dyn_into::<web_sys::HtmlCanvasElement>()?;
    let div = utils::dom::get_element("host-paint").unwrap();
    let _ = div.append_child(&canvas);
    canvas.set_id("draw-canvas");
    canvas.set_width(600);
    canvas.set_height(500);
    canvas.set_class_name("paint");
    let context = canvas
        .get_context("2d")?
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()?;
    
    context.set_font("20px Arial");
    draw_content(content, &context);

    if is_host {
        host_action(&canvas, context, send_message_all_cb);
    } else {
        // client_action(&canvas, context, send_message_all_cb);
    }
    
    Ok(Rc::new(canvas))
}

fn draw_content(content: &str, context: &web_sys::CanvasRenderingContext2d) {
    let arr = content.lines();
    let mut step = 20.0;
    arr.into_iter().for_each(|line| {
        let _ = context.fill_text(line, 10., step);
        step += 20.0;
    });
}

fn host_action(canvas: &web_sys::HtmlCanvasElement, context: web_sys::CanvasRenderingContext2d, send_message_all_cb: Callback<Message>) {
    let context = Rc::new(context);
    let pressed = Rc::new(Cell::new(false));
       
    {
        let context = context.clone();
        let pressed = pressed.clone();
        let send_message_all_cb = send_message_all_cb.clone();
        let closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::MouseEvent| {
            context.begin_path();
            context.move_to(event.offset_x() as f64, event.offset_y() as f64);
            pressed.set(true);
            send_message_all_cb.emit(
                Message::HostPaint { 
                    offset_x: event.offset_x() as f64,
                    offset_y: event.offset_y() as f64,
                    action: PaintAction::Down
                 }
            )
        });
        canvas.add_event_listener_with_callback("mousedown", closure.as_ref().unchecked_ref()).expect("error add event listener paint mousedown");
        closure.forget();
    }
    {
        let context = context.clone();
        let pressed = pressed.clone();
        let send_message_all_cb = send_message_all_cb.clone();
        let closure = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
            if pressed.get() {
                context.line_to(event.offset_x() as f64, event.offset_y() as f64);
                context.stroke();
                context.begin_path();
                context.move_to(event.offset_x() as f64, event.offset_y() as f64);
                send_message_all_cb.emit(
                    Message::HostPaint { 
                        offset_x: event.offset_x() as f64,
                        offset_y: event.offset_y() as f64,
                        action: PaintAction::Move
                     }
                )
            }
        }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("mousemove", closure.as_ref().unchecked_ref()).expect("error add event listener pain mousemove");
        closure.forget();
    }
    {
        let context = context.clone();
        let send_message_all_cb = send_message_all_cb.clone();
        let closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::MouseEvent| {
            context.line_to(event.offset_x() as f64, event.offset_y() as f64);
            context.stroke();
            pressed.set(false);
            send_message_all_cb.emit(
                Message::HostPaint { 
                    offset_x: event.offset_x() as f64,
                    offset_y: event.offset_y() as f64,
                    action: PaintAction::Up
                 }
            )
        });
        canvas.add_event_listener_with_callback("mouseup", closure.as_ref().unchecked_ref()).expect("error add event listener paint mouseup");
        closure.forget();
    }
}


fn client_action(canvas: &web_sys::HtmlCanvasElement, context: web_sys::CanvasRenderingContext2d, send_message_all_cb: Callback<Message>) {
    let context = Rc::new(context);
    let pressed = Rc::new(Cell::new(false));
       
    {
        let context = context.clone();
        let pressed = pressed.clone();
        let closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::MouseEvent| {
            context.begin_path();
            context.move_to(event.offset_x() as f64, event.offset_y() as f64);
            pressed.set(true);
        });
        canvas.add_event_listener_with_callback("mousedown", closure.as_ref().unchecked_ref()).expect("error add event listener paint mousedown");
        closure.forget();
    }
    {
        let context = context.clone();
        let pressed = pressed.clone();
        let closure = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
            if pressed.get() {
                context.line_to(event.offset_x() as f64, event.offset_y() as f64);
                context.stroke();
                context.begin_path();
                context.move_to(event.offset_x() as f64, event.offset_y() as f64);
                send_message_all_cb.emit(
                    Message::HostPaint { 
                        offset_x: event.offset_x() as f64,
                        offset_y: event.offset_y() as f64,
                        action: PaintAction::Move
                    }
                )
            }
        }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("mousemove", closure.as_ref().unchecked_ref()).expect("error add event listener pain mousemove");
        closure.forget();
    }
    {
        let context = context.clone();
        let closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::MouseEvent| {
            context.line_to(event.offset_x() as f64, event.offset_y() as f64);
            context.stroke();
            pressed.set(false);
        });
        canvas.add_event_listener_with_callback("mouseup", closure.as_ref().unchecked_ref()).expect("error add event listener paint mouseup");
        closure.forget();
    }
}

