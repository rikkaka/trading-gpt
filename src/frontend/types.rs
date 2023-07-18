#[derive(Clone)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

impl Message {
    pub fn new(role: Role, content: String) -> Message {
        Message { role, content }
    }
}

#[derive(Clone)]
pub enum Role {
    User,
    Bot,
}
