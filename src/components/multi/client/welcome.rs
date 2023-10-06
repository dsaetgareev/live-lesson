use std::{cell::RefCell, rc::Rc};

use yew::{Component, html};

use crate::{models::audio::Audio, components::multi::client::client::Client, utils::device::create_audio_decoder};

pub enum Msg {
    ToClient(Rc<RefCell<Audio>>)
}

pub struct Welcome {
    pub is_client: bool,
    audio: Option<Rc<RefCell<Audio>>>,
}

impl Component for Welcome {
    type Message = Msg;

    type Properties = ();

    fn create(_ctx: &yew::Context<Self>) -> Self {
        Self { 
            is_client: false,
            audio: None,
        }
    }

    fn update(&mut self, ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ToClient(audio) => {
                self.audio = Some(audio);
                self.is_client = true;
                true
            }
        }
    }

    fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
        let on_init = ctx.link().callback(|_| {
            let audio = Rc::new(RefCell::new(create_audio_decoder()));
            Msg::ToClient(audio)
        });
        
        html! {
            if !self.is_client {
                <button onclick={ on_init }>
                    { 
                        "Заходи дорогой!"
                    }
                    
                </button>
            } else {
                <Client />
            }
            
        }
    }
}