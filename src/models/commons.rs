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

#[derive(PartialEq)]
pub struct PaintProps {
    pub offset_x: f64,
    pub offset_y: f64
}

impl PaintProps {
    pub fn new() -> Self {
        Self { 
            offset_x: f64::default(),
            offset_y: f64::default()
        }
    }

    pub fn set_offset_x(&mut self, x: f64) {
        self.offset_x = x;
    }

    pub fn set_offset_y(&mut self, y: f64) {
        self.offset_y = y;
    }
}