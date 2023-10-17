use yew::Properties;

use super::commons::{AreaKind, TextAreaProps, PaintProps};


#[derive(Clone, PartialEq, Properties)]
pub struct ClientProps {
    pub client_editor_content: String,
    pub client_text_area: TextAreaProps,
    pub client_id: String,
    pub is_write: bool,
    pub client_area_kind: AreaKind,
    pub paint_props: PaintProps,
}

impl ClientProps {
    pub fn new() -> Self {
        Self { 
            client_editor_content: String::default(),
            client_text_area: TextAreaProps::new(),
            client_id: String::default(),
            is_write: false,
            client_area_kind: AreaKind::TextArea,
            paint_props: PaintProps::new(),  
         }
    }

    pub fn set_editor_content(&mut self, content: String) {
        self.client_editor_content = content;
    }

    pub fn set_text_area_content(&mut self, content: String) {
        self.client_text_area.set_content(content);
    }

    pub fn set_area_kind(&mut self, area_kind: AreaKind) {
        self.client_area_kind = area_kind;
    }

    pub fn set_client_id(&mut self, client_id: String) {
        self.client_id = client_id;
    }

    pub fn set_is_write(&mut self, is_write: bool) {
        self.is_write = is_write;
    }
}

#[derive(PartialEq, Clone, Properties)]
pub struct ClientItem {
    pub editor_content: String,
    pub text_area_content: String,
    pub area_kind: AreaKind,
}

impl ClientItem {
    pub fn new(area_kind: AreaKind) -> Self {
        Self { 
            editor_content: String::default(), 
            text_area_content: String::default(),
            area_kind,
        }
    }

    pub fn set_editor_content(&mut self, content: String) {
        self.editor_content = content;
    }

    pub fn set_text_area_content(&mut self, content: String) {
        self.text_area_content = content;
    }

    pub fn set_area_kind(&mut self, area_kind: AreaKind) {
        self.area_kind = area_kind;
    }
}
