use diesel::prelude::*;

#[derive(Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = super::schema::users)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[derive(Debug, PartialEq, Eq)]
pub struct User {
    pub username: String,
    pub password: String,
    pub balance: i32,
}

impl User {
    pub fn new(username: String, password: String, balance: i32) -> User {
        User {
            username,
            password,
            balance,
        }
    }
}
