use diesel::prelude::*;
use diesel::pg::PgConnection;
use multimap::MultiMap;
use std::collections::HashMap;
use std::iter::*;
use std::time::SystemTime;

use super::*;
use super::models::*;
use super::schema;
use super::wikidot::Wikidot;

fn drain_filter<F, T>(vec: &mut Vec<T>, filter: F) -> Vec<T> where F: Fn(&T) -> bool {
    let mut drained = Vec::new();
    let mut i = 0;
    while i != vec.len() {
        if filter(&vec[i]) {
            drained.push(vec.remove(i));
        } else {
            i += 1;
        }
    }
    drained
}

pub struct Db {
    nick:       String,
    conn:       PgConnection,
    properties: HashMap<String, String>,
    reminders:  MultiMap<String, Reminder>,
    users:      HashMap<String, User>,
    pub wiki:   Option<Wikidot>
}

impl Db {
    pub fn new() -> Db {
        let conn = establish_connection();
        Db {
            nick:       from_env("IRC_NICK").to_lowercase(),
            properties: load_properties(&conn), 
            reminders:  load_reminders(&conn),
            users:      load_users(&conn), 
            wiki:       Wikidot::new(), 
            conn 
        }
    }

    pub fn reload(&mut self) {
        self.properties = load_properties(&self.conn);
        self.reminders  = load_reminders(&self.conn);
        self.users      = load_users(&self.conn);
    }

    fn get_auth(&self, user: &str) -> i32 {
        let lower = user.to_lowercase();
        if lower == self.nick {
            5
        } else { 
            self.users.get(&lower).map(|x| x.auth).unwrap_or(0) 
        }
    }

    pub fn auth(&self, level: i32, user: &str) -> bool {
        level <= self.get_auth(user)
    }

    pub fn outranks(&self, x: &str, y: &str) -> bool {
        self.get_auth(x) > self.get_auth(y)
    }

    pub fn add_user(&mut self, auth: i32, nick_up: &str) -> Result<(), diesel::result::Error> {
        let nick = nick_up.to_lowercase();
        let user = User {
            nick: nick.to_owned(),
            auth,
            pronouns: self.users.get(&nick).and_then(|x| x.pronouns.to_owned())
        };
        diesel::insert_into(schema::user::table)
            .values(&user)
            .on_conflict(schema::user::nick)
            .do_update()
            .set(schema::user::auth.eq(auth))
            .execute(&self.conn)?;
        self.users.insert(nick, user);
        Ok(())
    }

    pub fn delete_user(&mut self, nick_up: &str) -> Result<bool, diesel::result::Error> {
        let nick = nick_up.to_lowercase();
        let removed = self.users.remove(&nick);
        diesel::delete(schema::user::table.filter(schema::user::nick.eq(nick)))
            .execute(&self.conn)?;
        Ok(removed.is_some())
    }

    pub fn add_reminder(&mut self, nick_up: &str, when: SystemTime, message: &str) 
    -> Result<(), diesel::result::Error>{
        let reminder = DbReminder {
            nick:    nick_up.to_lowercase(),
            when:    when,
            message: message.to_owned()
        };
        diesel::insert_into(schema::reminder::table)
            .values(&reminder)
            .execute(&self.conn)?;
        Ok(())
    }
    pub fn get_reminders(&mut self, nick_up: &str) -> Option<Vec<Reminder>> {
        let when = SystemTime::now();
        let mut reminders = self.reminders.remove(&nick_up.to_lowercase())?;
        let expired = drain_filter(&mut reminders, |x| x.when < when);
        for reminder in reminders {
            self.reminders.insert(reminder.nick.to_owned(), reminder);
        }
        diesel::delete(schema::reminder::table.filter(schema::reminder::when.lt(when)))
            .execute(&self.conn).ok();
        Some(expired)
    }
}

fn establish_connection() -> PgConnection {
    let database_url = from_env("DATABASE_URL");
    PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

fn load_properties(conn: &PgConnection) -> HashMap<String, String> {
    HashMap::from_iter(
        schema::property::table.load(conn)
            .expect("Error loading properties")
            .into_iter()
            .map(|x: Property| (x.key, x.value))
    )
}

fn load_reminders(conn: &PgConnection) -> MultiMap<String, Reminder> {
    MultiMap::from_iter(
        schema::reminder::table.load(conn)
            .expect("Error loading reminders")
            .into_iter()
            .map(|x: Reminder| (x.nick.to_owned(), x))
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
