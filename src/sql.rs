use anyhow::Result;
use dotenvy::dotenv;
use lazy_static::lazy_static;

use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};

use crate::schema::users;
use crate::types::User;

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
        self.insert_into_db_conn(&mut conn)
    }

    pub fn retrieve_from_db(username: &str) -> Result<User> {
        let mut conn = POOL.get()?;
        User::retrieve_from_db_conn(username, &mut conn)
    }

    pub fn update_to_db(&self) -> Result<()> {
        let mut conn = POOL.get()?;
        self.update_to_db_conn(&mut conn)
    }

    pub fn delete_from_db(&self) -> Result<()> {
        let mut conn = POOL.get()?;
        self.delete_from_db_conn(&mut conn)
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
}

fn count_users(conn: &mut Pooled) -> Result<i64> {
    let count = users::table.count().get_result(conn)?;
    Ok(count)
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
        let count = count_users(&mut conn).unwrap();
        assert_eq!(count, 0);
    }
}
