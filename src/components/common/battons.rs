use web_sys::MouseEvent;
use yew::{Html, html, function_component, Callback, Properties};
use yew_icons::{Icon, IconId};

#[derive(PartialEq, Properties)]
pub struct VideoButtonProps {
    pub on_video_btn: Callback<MouseEvent>,
    pub enabled: bool,
}

#[function_component(VideoButton)]
pub fn video_btn(props: &VideoButtonProps) -> Html {
    
    html! {
        <div>
            <button onclick={ props.on_video_btn.clone() }>
                { 
                    if props.enabled {
                        html! { <Icon icon_id={IconId::BootstrapCameraVideoOffFill}/> }
                    } else {
                        html! { <Icon icon_id={IconId::BootstrapCameraVideoFill}/> }
                    }
                }
            </button>
        </div>
    }
}