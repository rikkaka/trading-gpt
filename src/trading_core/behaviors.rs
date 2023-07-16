use super::types::User;
use crate::global;

use anyhow::Result;

impl User {
    pub fn signup(username: String, password: String) -> Result<User> {
        let user = User::new(username, password, global::START_MONEY);
        user.init()?;
        Ok(user)
    }

    pub fn login(username: String, passsword: String) -> Result<User> {
        unimplemented!()
    }

    pub fn logout(&self) -> Result<()> {
        unimplemented!()
    }

    pub fn transfer(&self, to: &User, amount: u32) -> Result<()> {
        unimplemented!()
    }

    fn init(&self) -> Result<()> {
        self.insert_into_db()
    }

    fn login_still_valid(&self) -> Result<()> {
        unimplemented!()
    }
}
