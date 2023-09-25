use serde::{Deserialize, Serialize};


#[derive(PartialEq, Clone, Copy, Serialize, Deserialize, Debug)]
pub enum AreaKind {
    Editor,
    TextArea
}

#[derive(PartialEq)]
pub struct TextAreaProps {
    pub content: String,
    pub placeholder: String,
    pub is_disabled: bool,
}

impl TextAreaProps {
    pub fn new() -> Self {
        Self {
            content: String::default(),
            placeholder: String::default(),
            is_disabled: true
        }
    }

    pub fn set_content(&mut self, content: String) {
        self.content = content;
    }

    pub fn set_placeholder(&mut self, placeholder: String) {
        self.placeholder = placeholder;
    }
}