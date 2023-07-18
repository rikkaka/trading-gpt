use super::types::User;
use crate::global;

use anyhow::{anyhow, bail, Result};

impl User {
    pub fn signup(username: &str, password: &str) -> Result<User> {
        let user = User::new(
            username.to_string(),
            password.to_string(),
            global::START_MONEY,
        );
        user.init()?;
        Ok(user)
    }

    pub fn login(username: &str, passsword: &str) -> Result<User> {
        let user = User::retrieve_from_db(username)?;
        user.check_password(passsword)?;
        Ok(user)
    }

    pub fn logout(&self) -> Result<()> {
        unimplemented!()
    }

    pub fn transfer(&mut self, to: &str, amount: i32) -> Result<()> {
        self.transfer_to_other(to, amount)
    }

    fn init(&self) -> Result<()> {
        self.insert_into_db()
    }

    fn login_still_valid(&self) -> Result<()> {
        unimplemented!()
    }

    fn check_password(&self, password: &str) -> Result<()> {
        if self.password == password {
            Ok(())
        } else {
            bail!("Wrong password")
        }
    }
}
