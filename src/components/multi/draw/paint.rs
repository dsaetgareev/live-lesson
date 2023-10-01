use std::cell::Cell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;
use yew::{Callback, Component, Properties, NodeRef, html};

use crate::utils;
use crate::utils::inputs::Message;

pub enum Msg {

}

#[derive(PartialEq, Properties)]
pub struct CurrentProps {
    #[prop_or_default]
    pub content: String,
    #[prop_or_default]
    pub send_message_all_cb: Callback<Message>,
}


pub struct Paint {
    canvas: NodeRef,
    send_message_all_cb: Callback<Message>,
}

impl Paint {

    pub fn send_message_to_all(&self, message: Message) {
        self.send_message_all_cb.emit(message);
    }

    fn get_context(&self) -> web_sys::CanvasRenderingContext2d {
        let context = self.canvas
                .cast::<HtmlCanvasElement>()
                .expect("cannot cast element")
                .get_context("2d")
                .expect("cannot get context")
                .unwrap()
                .dyn_into::<web_sys::CanvasRenderingContext2d>()
                .expect("cannot get context");
            
            context.set_font("20px Arial");
        context
    }
        
    fn host_action(&mut self) {
        let canvas = self.canvas
            .cast::<HtmlCanvasElement>()
            .expect("cannot cast element");
        let context = Rc::new(self.get_context());
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
            let send_message_to_all = self.send_message_all_cb.clone();
            let closure = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
                if pressed.get() {
                    context.line_to(event.offset_x() as f64, event.offset_y() as f64);
                    context.stroke();
                    context.begin_path();
                    context.move_to(event.offset_x() as f64, event.offset_y() as f64);
                    send_message_to_all.emit(
                        Message::HostPaint { offset_x: event.offset_x() as f64, offset_y: event.offset_y() as f64 }
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
}

impl Component for Paint {
    type Message = ();

    type Properties = CurrentProps;

    fn create(ctx: &yew::Context<Self>) -> Self {
        Self { 
            canvas: NodeRef::default(),
            send_message_all_cb: ctx.props().send_message_all_cb.clone()
        }
    }

    fn rendered(&mut self, ctx: &yew::Context<Self>, first_render: bool) {
        if first_render {
            let canvas = self.canvas
                .cast::<HtmlCanvasElement>()
                .expect("cannot get canvas element");

            let div = utils::dom::get_element("host-host").unwrap();
            let _ = div.append_child(&canvas);
            canvas.set_width(640);
            canvas.set_height(480);
            canvas.set_class_name("paint");
            let context = canvas
                .get_context("2d")
                .expect("cannot get context")
                .unwrap()
                .dyn_into::<web_sys::CanvasRenderingContext2d>()
                .expect("cannot get context");
            
            context.set_font("20px Arial");
            draw_content(&ctx.props().content, &context);
            self.host_action();
        }
    }

    fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
        html! {
            <div>
                <canvas ref={ self.canvas.clone() } class="paint"></canvas>
            </div>
        }
    }
}

// #[wasm_bindgen(start)]
pub fn start(
    content: &str,
    send_message_all_cb: Callback<Message>
) -> Result<(), JsValue> {
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document
        .create_element("canvas")?
        .dyn_into::<web_sys::HtmlCanvasElement>()?;
    let div = utils::dom::get_element("host-host").unwrap();
    let _ = div.append_child(&canvas);
    canvas.set_width(640);
    canvas.set_height(480);
    canvas.set_class_name("paint");
    let context = canvas
        .get_context("2d")?
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()?;
    
    context.set_font("20px Arial");
    draw_content(content, &context);
    host_action(&canvas, context, send_message_all_cb);
    Ok(())
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
                    Message::HostPaint { offset_x: event.offset_x() as f64, offset_y: event.offset_y() as f64 }
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
                    Message::HostPaint { offset_x: event.offset_x() as f64, offset_y: event.offset_y() as f64 }
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

