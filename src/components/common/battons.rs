use web_sys::MouseEvent;
use yew::{Html, html, function_component, Callback, Properties, use_state};
use yew_icons::{Icon, IconId};

#[derive(PartialEq, Properties)]
pub struct ButtonProps {
    pub on_btn: Callback<MouseEvent>,
    pub enabled: bool,
}

#[function_component(VideoButton)]
pub fn video_btn(props: &ButtonProps) -> Html {
    let enabled = use_state(|| props.enabled);
    
    html! {
        <div>
            <button onclick={ props.on_btn.clone() }>
                { 
                    if *enabled {
                        html! { <Icon icon_id={IconId::BootstrapCameraVideoOffFill}/> }
                    } else {
                        html! { <Icon icon_id={IconId::BootstrapCameraVideoFill}/> }
                    }
                }
            </button>
        </div>
    }
}

#[function_component(AudioButton)]
pub fn audio_btn(props: &ButtonProps) -> Html {
    
    html! {
        <div>
            <button onclick={ props.on_btn.clone() }>
                { 
                    if props.enabled {
                        html! { <Icon icon_id={IconId::BootstrapMicMuteFill}/> }
                    } else {
                        html! { <Icon icon_id={IconId::BootstrapMicFill}/> }
                    }
                }
            </button>
        </div>
    }
}