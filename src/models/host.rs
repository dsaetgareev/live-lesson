use crate::models::commons::TextAreaProps;

use super::commons::AreaKind;

#[derive(PartialEq)]
pub struct HostPorps {
    pub host_editor_content: String,
    pub host_area_content: TextAreaProps,
    pub host_area_kind: AreaKind,
}

impl HostPorps {
    pub fn new() -> Self {
        Self {
            host_editor_content: String::default(),
            host_area_content: TextAreaProps::new(),
            host_area_kind: AreaKind::TextArea,
        }
    }
}
