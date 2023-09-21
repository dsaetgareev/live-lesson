use monaco::{
    api::{CodeEditorOptions, TextModel},
    sys::{editor::{BuiltinTheme, IStandaloneCodeEditor}, Position, IPosition},
    yew::{CodeEditor, CodeEditorLink},
};
use wasm_bindgen::closure::Closure;
use yew::prelude::*;

use wasm_bindgen::JsCast;

const CONTENT: &str = include_str!("editor.rs");

fn get_options() -> CodeEditorOptions {
    CodeEditorOptions::default()
        .with_language("rust".to_owned())
        // .with_value(content)
        .with_builtin_theme(BuiltinTheme::VsDark)
        .with_automatic_layout(true)
}

#[derive(PartialEq, Properties)]
pub struct CustomEditorProps {
    #[prop_or_default]
    pub on_editor_created: Callback<CodeEditorLink>,
    pub text_model: TextModel,
}

#[derive(PartialEq, Properties)]
pub struct EditorWrapperProps {
    #[prop_or_default]
    pub on_cb: Callback<String>,
    pub text_model: TextModel,
    pub is_write: bool,
}

///
/// This is really just a helper component, so we can pass in props easier.
/// It makes it much easier to use, as we can pass in what we need, and it
/// will only re-render if the props change.
///
#[function_component(CustomEditor)]
pub fn custom_editor(props: &CustomEditorProps) -> Html {
    let CustomEditorProps {
        on_editor_created,
        text_model,
    } = props;

    html! {
        <CodeEditor classes={"full-height"} options={ get_options().to_sys_options() } {on_editor_created} model={text_model.clone()} />
    }
}

#[function_component(EditorWrapper)]
pub fn editor_wrapper(props: &EditorWrapperProps) -> Html {
    // We need to create a new text model, so we can pass it to Monaco.
    // We use use_state_eq, as this allows us to only use it when it changes.
    let text_model = use_state_eq(|| props.text_model.clone());
    // let text_model = props.text_model.clone();
    if props.is_write {
        text_model.set_value(&props.text_model.get_value());
    }    
    log::error!("text_model {}", text_model.get_value());

    let on_cb = &props.on_cb;

    // Here we setup the Callback for when the editor is created.
    let on_editor_created = {
        // We need to clone the text_model/code so we can use them.
        let text_model = text_model.clone();

        // This is a javascript closure, used to pass to Monaco, using wasm-bindgen.
        let js_closure = {
            let text_model = text_model.clone();
            let on_cb = on_cb.clone();
            // We update the code state when the Monaco model changes.
            // See https://yew.rs/docs/0.20.0/concepts/function-components/pre-defined-hooks
            Closure::<dyn Fn()>::new(move || {
                on_cb.emit(text_model.get_value());
            })
        };

        // Here we define our callback, we use use_callback as we want to re-render when dependencies change.
        // See https://yew.rs/docs/concepts/function-components/state#general-view-of-how-to-store-state
        use_callback(
            move |editor_link: CodeEditorLink, _text_model| {
                editor_link.with_editor(|editor| {
                    let raw_editor: &IStandaloneCodeEditor = editor.as_ref();
                    raw_editor.on_key_up(js_closure.as_ref().unchecked_ref());          
                });
            },
            text_model,
        )
    };
    html! {
        <div class="code-wrapper">
            <CustomEditor {on_editor_created} text_model={(*text_model).clone()} />
        </div>
    }
}

