pub struct Bot {}

impl Bot {
    pub fn new() -> Bot {
        Bot {}
    }

    pub async fn chat(&mut self, draft: &str) -> String {
        "developing...".into()
    }
}
