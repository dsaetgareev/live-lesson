

pub struct HostPorps {
    pub host_content: String,
}

impl HostPorps {
    pub fn new() -> Self {
        Self {
            host_content: String::default(),
        }
    }
}