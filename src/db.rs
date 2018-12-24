use diesel::prelude::*;
use diesel::pg::PgConnection;
use dotenv::dotenv;
use std::env;
use std::collections::HashMap;
use std::iter::*;

use super::models::*;
use super::schema;
use super::wikidot::Wikidot;

pub struct Db {
    conn: PgConnection,
    pub props: HashMap<String, String>,
    pub users: HashMap<String, User>,
    pub wiki: Wikidot
}

impl Db {
    pub fn new() -> Db {
        let conn = establish_connection();
        let props = load_props(&conn);
        let users = load_users(&conn);
        let wiki = load_wiki(&props);
        Db { conn, props, users, wiki }
    }

    pub fn auth(&self, level: i32, user: &str) -> bool {
        match self.users.get(&user.to_lowercase()) {
            None => false,
            Some(user) => user.auth >= level
        }
    }

    fn reload(&mut self) {
        self.props = load_props(&self.conn);
        self.users = load_users(&self.conn);
        self.wiki = load_wiki(&self.props);
    }
}

fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
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

fn load_wiki(props: &HashMap<String, String>) -> Wikidot {
    Wikidot::new(
        props.get("wikidotUser").unwrap(), 
        props.get("wikidotKey").unwrap()
    )
}
