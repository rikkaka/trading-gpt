pub struct Bot {}

impl Bot {
    pub fn new() -> Bot {
        Bot {}
    }

    pub fn chat(&mut self, draft: &str) -> String {
        "developing...".into()
    }
}
