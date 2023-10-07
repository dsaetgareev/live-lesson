use crate::models::commons::TextAreaProps;

use super::commons::AreaKind;

#[derive(PartialEq)]
pub struct HostPorps {
    pub host_editor_content: String,
    pub host_area_content: TextAreaProps,
    pub host_area_kind: AreaKind,
    pub is_communication: bool,
}

impl HostPorps {
    pub fn new() -> Self {
        Self {
            host_editor_content: String::default(),
            host_area_content: TextAreaProps::new(),
            host_area_kind: AreaKind::Editor,
            is_communication: true,
        }
    }

    pub fn set_host_area_kind(&mut self, host_area_kind: AreaKind) {
        self.host_area_kind = host_area_kind;
    }

    pub fn set_editor_content(&mut self, content: String) {
        self.host_editor_content = content;
    }

    pub fn set_text_area_content(&mut self, content: String) {
        self.host_area_content.set_content(content);
    }

    pub fn is_communication(&mut self, is_communication: bool) {
        self.is_communication = is_communication;
    }

    pub fn switch_communication(&mut self) -> bool {
        self.is_communication = !self.is_communication;
        self.is_communication
    }
}
