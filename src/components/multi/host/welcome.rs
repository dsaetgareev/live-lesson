use std::{cell::RefCell, rc::Rc};

use yew::{Component, html};

use crate::{models::audio::Audio, components::multi::host::host::Host, utils::device::create_audio_decoder};

pub enum Msg {
    ToClient(Rc<RefCell<Audio>>)
}

pub struct WelcomeHost {
    pub is_host: bool,
    audio: Option<Rc<RefCell<Audio>>>,
}

impl Component for WelcomeHost {
    type Message = Msg;

    type Properties = ();

    fn create(_ctx: &yew::Context<Self>) -> Self {
        Self { 
            is_host: false,
            audio: None,
        }
    }

    fn update(&mut self, _ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ToClient(audio) => {
                self.audio = Some(audio);
                self.is_host = true;
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
            if !self.is_host {
                <button onclick={ on_init }>
                    { 
                        "Заходи дорогой!"
                    }
                    
                </button>
            } else {
                <Host />
            }
            
        }
    }
}