use anyhow::{anyhow, bail, Error, Result};
use dotenvy::dotenv;
use lazy_static::lazy_static;

use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};

use super::schema::users;
use super::types::User;

lazy_static! {
    static ref POOL: Pool<ConnectionManager<SqliteConnection>> = {
        dotenv().ok();

        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL hasn't been set");
        let manager = ConnectionManager::<SqliteConnection>::new(database_url);
        Pool::builder()
            .build(manager)
            .expect("Failed to create pool.")
    };
}

type Pooled = diesel::r2d2::PooledConnection<ConnectionManager<SqliteConnection>>;

impl User {
    pub fn insert_into_db(&self) -> Result<()> {
        let mut conn = POOL.get()?;
        if User::check_existence_conn(&self.username, &mut conn)? {
            bail!("Username already exists")
        }
        self.insert_into_db_conn(&mut conn)
    }

    pub fn retrieve_from_db(username: &str) -> Result<User> {
        let mut conn = POOL.get()?;
        if !User::check_existence_conn(username, &mut conn)? {
            bail!("Username doesn't exist")
        }
        User::retrieve_from_db_conn(&username, &mut conn)
    }

    pub fn update_to_db(&self) -> Result<()> {
        let mut conn = POOL.get()?;
        self.update_to_db_conn(&mut conn)
    }

    pub fn delete_from_db(&self) -> Result<()> {
        let mut conn = POOL.get()?;
        self.delete_from_db_conn(&mut conn)
    }

    pub fn transfer_to_other(&mut self, to_username: &str, amount: i32) -> Result<()> {
        let mut conn = POOL.get().unwrap();
        conn.transaction::<_, Error, _>(|conn| {
            self.check_balance_conn(amount, conn)?;
            let mut other = User::retrieve_from_db_conn(to_username, conn)?;
            self.balance -= amount;
            self.update_to_db_conn(conn).unwrap();
            other.balance += amount;
            other.update_to_db_conn(conn).unwrap();

            Ok(())
        })
    }

    fn check_existence_conn(username: &str, conn: &mut Pooled) -> Result<bool> {
        let count = users::table
            .filter(users::username.eq(username))
            .count()
            .get_result::<i64>(conn)?;
        Ok(count > 0)
    }

    fn insert_into_db_conn(&self, conn: &mut Pooled) -> Result<()> {
        diesel::insert_into(users::table)
            .values(self)
            .execute(conn)?;
        Ok(())
    }

    fn retrieve_from_db_conn(username: &str, conn: &mut Pooled) -> Result<User> {
        let user = users::table
            .filter(users::username.eq(username))
            .first::<User>(conn)?;
        Ok(user)
    }

    fn update_to_db_conn(&self, conn: &mut Pooled) -> Result<()> {
        diesel::update(users::table)
            .filter(users::username.eq(&self.username))
            .set(self)
            .execute(conn)?;
        Ok(())
    }

    fn delete_from_db_conn(&self, conn: &mut Pooled) -> Result<()> {
        diesel::delete(users::table)
            .filter(users::username.eq(&self.username))
            .execute(conn)?;
        Ok(())
    }

    fn check_balance_conn(&self, amount: i32, conn: &mut Pooled) -> Result<()> {
        let balance = users::table
            .filter(users::username.eq(&self.username))
            .select(users::balance)
            .first::<i32>(conn)?;
        if balance >= amount {
            Ok(())
        } else {
            bail!("Insufficient balance")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool() {
        POOL.get().unwrap();
    }

    #[test]
    fn test_crud() {
        let mut conn = POOL.get().unwrap();
        conn.begin_test_transaction().unwrap();

        let user = User::new("test".to_string(), "test".to_string(), 100);
        user.insert_into_db_conn(&mut conn).unwrap();
        let user2 = User::retrieve_from_db_conn("test", &mut conn).unwrap();
        assert_eq!(user, user2);

        let user_updated = User::new("test".to_string(), "test2".to_string(), 100);
        user_updated.update_to_db_conn(&mut conn).unwrap();
        let user3 = User::retrieve_from_db_conn("test", &mut conn).unwrap();
        assert_eq!(user_updated, user3);

        user3.delete_from_db_conn(&mut conn).unwrap();
        let count: i64 = users::table.count().get_result(&mut conn).unwrap();
        assert_eq!(count, 0);
    }
}
