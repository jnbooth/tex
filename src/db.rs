use diesel::prelude::*;
use diesel::pg::PgConnection;
use std::collections::HashMap;
use std::iter::*;

use super::*;
use super::models::*;
use super::schema;
use super::wikidot::Wikidot;

pub struct Db {
    conn: PgConnection,
    pub props: HashMap<String, String>,
    pub users: HashMap<String, User>,
    pub wiki: Option<Wikidot>
}

impl Db {
    pub fn new() -> Db {
        let conn = establish_connection();
        Db { props: load_props(&conn), users: load_users(&conn), wiki: Wikidot::new(), conn }
    }

    pub fn auth(&self, level: i32, user: &str) -> bool {
        match self.users.get(&user.to_lowercase()) {
            None => false,
            Some(user) => user.auth >= level
        }
    }

    pub fn reload(&mut self) {
        self.props = load_props(&self.conn);
        self.users = load_users(&self.conn);
    }
}

fn establish_connection() -> PgConnection {
    let database_url = from_env("DATABASE_URL");
    PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

fn load_props(conn: &PgConnection) -> HashMap<String, String> {
    HashMap::from_iter(
        schema::property::table.load(conn)
            .expect("Error loading properties")
            .into_iter()
            .map(|x: Property| (x.key, x.value))
    )
}

fn load_users(conn: &PgConnection) -> HashMap<String, User> {
    HashMap::from_iter(
        schema::user::table.load(conn)
            .expect("Error loading users")
            .into_iter()
            .map(|x: User| (x.nick.to_owned(), x))
    )   
}
