use diesel::prelude::*;
use diesel::pg::PgConnection;
use dotenv::dotenv;
use std::env;
use std::collections::HashMap;
use std::iter::*;

use super::models;
use super::schema;

pub struct Db {
    conn: PgConnection,
    pub props: HashMap<String, String>,
    pub users: HashMap<String, models::User>
}

impl Db {
    pub fn new() -> Db {
        let mut db = 
                Db { conn: establish_connection(), props: HashMap::new(), users: HashMap::new() };
        db.load();
        db
    }

    pub fn auth(&self, level: i32, user: &str) -> bool {
        match self.users.get(&user.to_lowercase()) {
            None => false,
            Some(user) => user.auth >= level
        }
    }

    pub fn load(&mut self) {
        self.props = HashMap::from_iter(
            schema::property::table.load(&self.conn)
                .expect("Error loading properties")
                .into_iter()
                .map(|x: models::Property| (x.key, x.value))
        );
        self.users = HashMap::from_iter(
            schema::user::table.load(&self.conn)
                .expect("Error loading users")
                .into_iter()
                .map(|x: models::User| (x.nick.to_owned(), x))
        );
    }

}

fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}
