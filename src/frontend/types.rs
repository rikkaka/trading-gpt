#[derive(Clone)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

impl Message {
    pub fn new(role: Role, content: String) -> Message {
        Message {
            role,
            content,
        }
    }

    pub fn update_content(&mut self, content: String) {
        self.content = content;
    }
}

#[derive(Clone)]
pub enum Role {
    User,
    Bot,
}