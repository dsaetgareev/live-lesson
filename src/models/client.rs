
pub struct ClientProps {
    pub client_content: String,
    pub client_id: String,
    pub is_write: bool,
}

impl ClientProps {
    pub fn new(
        client_content: String,
        client_id: String,
    ) -> Self {
        Self { 
            client_content,
            client_id,
            is_write: false    
         }
    }
}
