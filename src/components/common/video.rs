use yew::{Html, html, function_component, Properties};
use yew_icons::{Icon, IconId};

#[derive(PartialEq, Properties)]
pub struct VideoBoxProps {
    pub video_id: String,
    pub video_class: String,
    pub placeholder_id: String,
    pub placeholder_class: String,
}

#[function_component(VideoBox)]
pub fn video_box(props: &VideoBoxProps) -> Html {

    html! {
        <>
            <video class={ props.video_class.clone() } autoplay=true id={ props.video_id.clone() } poster="placeholder.png"></video>
            <div id={ props.placeholder_id.clone() } class={ props.placeholder_class.clone() }>
                <Icon icon_id={IconId::FontAwesomeSolidHorseHead}/>
            </div>
        </>
        
    }
}