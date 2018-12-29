use core::hash::Hash;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use multimap::MultiMap;
use std::collections::HashMap;
use std::iter::*;
use std::time::SystemTime;

use super::{Api, from_env, from_env_opt, from_env_api};
use super::color;
use super::models::*;
use super::response::choice::Choices;
use super::wikidot::Wikidot;

pub fn log<T>(res: QueryResult<T>) {
    if let Err(e) = res {
        color::log(color::WARN, &format!("DB error: {}", e));
    }
}

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

fn multi_remove<K: Eq + Hash, V: Eq>(map: &mut MultiMap<K, V>, k: K, v: V) -> bool {
    if let Some(vec) = map.get_vec_mut(&k) {
        let mut i = 0;
        while i != vec.len() {
            if vec[i] == v {
                vec.remove(i);
                return true
            } else {
                i += 1;
            }
        }
    }
    false
}

pub struct Apis {
    pub google:  Option<Api>,
    pub wikidot: Option<Wikidot>
}

impl Apis {
    pub fn new() -> Apis {
        Apis {
            google:  from_env_api("GOOGLE", "CUSTOMENGINE", "KEY"),
            wikidot: from_env_api("WIKIDOT", "USER", "KEY").map(Wikidot::new)
        }
    }
}

pub struct Db {
    pub choices: Choices,
    conn:        PgConnection,
    pub client:  reqwest::Client,
    nick:        String,
    owner:       Option<String>,
    properties:  HashMap<String, String>,
    reminders:   MultiMap<String, Reminder>,
    silences:    MultiMap<String, String>,
    users:       HashMap<String, User>,
    pub api:     Apis
}

impl Db {
    pub fn new() -> Db {
        let conn = establish_connection();
        Db {
            choices:    Choices::new(),
            client:     reqwest::Client::new(),
            nick:       from_env("IRC_NICK").to_lowercase(),
            owner:      from_env_opt("OWNER").map(|x| x.to_lowercase()),
            properties: load_properties(&conn),
            reminders:  load_reminders(&conn),
            silences:   load_silences(&conn),
            users:      load_users(&conn),
            api:        Apis::new(),
            conn
        }
    }

    pub fn reload(&mut self) {
        self.properties = load_properties(&self.conn);
        self.reminders  = load_reminders(&self.conn);
        self.silences   = load_silences(&self.conn);
        self.users      = load_users(&self.conn);
    }

    fn get_auth(&self, user: &str) -> i32 {
        let lower = user.to_lowercase();
        if self.nick == lower {
            5
        } else if self.owner == Some(lower.to_owned()) {
            4
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

    pub fn add_user(&mut self, auth: i32, nick_up: &str) -> QueryResult<()> {
        use super::schema::user;
        let nick = nick_up.to_lowercase();
        let user = User {
            nick: nick.to_owned(),
            auth,
            pronouns: self.users.get(&nick).and_then(|x| x.pronouns.to_owned())
        };
        diesel::insert_into(user::table)
            .values(&user)
            .on_conflict(user::nick)
            .do_update()
            .set(user::auth.eq(auth))
            .execute(&self.conn)?;
        self.users.insert(nick, user);
        Ok(())
    }

    pub fn delete_user(&mut self, nick_up: &str) -> QueryResult<bool> {
        use super::schema::user;
        let nick = nick_up.to_lowercase();
        let removed = self.users.remove(&nick);
        diesel::delete(user::table.filter(user::nick.eq(nick)))
            .execute(&self.conn)?;
        Ok(removed.is_some())
    }

    pub fn add_seen(&mut self, nick_up: &str, message: &str) -> QueryResult<()> {
        use super::schema::seen;
        let nick = nick_up.to_lowercase();
        let when = SystemTime::now();
        let seen = Seen { 
            nick, 
            first:  message.to_owned(), first_time:  when, 
            latest: message.to_owned(), latest_time: when,
            total:  1 
        };
        diesel::insert_into(seen::table)
            .values(&seen)
            .on_conflict(seen::nick)
            .do_update()
            .set((
                seen::latest.eq(message),
                seen::latest_time.eq(when),
                seen::total.eq(seen::total + 1)
            ))
            .execute(&self.conn)?;
        Ok(())
    }

    pub fn get_seen(&self, nick_up: &str) -> Option<Seen> {
        use super::schema::seen;
        let nick = nick_up.to_lowercase();
        seen::table
            .filter(seen::nick.eq(nick))
            .limit(1)
            .load(&self.conn)
            .expect("Error loading seen messages")
            .pop()
    }

    pub fn add_reminder(&mut self, nick_up: &str, when: SystemTime, message: &str) 
    -> QueryResult<()> {
        use super::schema::reminder;
        let reminder = DbReminder {
            nick:    nick_up.to_lowercase(),
            when:    when,
            message: message.to_owned()
        };
        diesel::insert_into(reminder::table)
            .values(&reminder)
            .execute(&self.conn)?;
        Ok(())
    }
    pub fn get_reminders(&mut self, nick_up: &str) -> Option<Vec<Reminder>> {
        use super::schema::reminder;
        let when = SystemTime::now();
        let mut reminders = self.reminders.get_vec_mut(&nick_up.to_lowercase())?;
        let expired = drain_filter(&mut reminders, |x| x.when < when);
        diesel::delete(reminder::table.filter(reminder::when.lt(when)))
            .execute(&self.conn).ok();
        Some(expired)
    }

    pub fn silenced(&self, channel: &str, command: &str) -> bool {
        match self.silences.get_vec(channel) {
            None => false,
            Some(silence) => silence.contains(&command.to_owned())
        }
    }

    pub fn set_enabled(&mut self, channel: &str, command: &str, enabled: bool) -> QueryResult<()> {
        use super::schema::silence;
        if enabled {
            multi_remove(&mut self.silences, channel.to_owned(), command.to_owned());
            diesel::delete(silence::table
                .filter(silence::channel.eq(channel))
                .filter(silence::command.eq(command))
            ).execute(&self.conn)?;
        } else {
            self.silences.insert(channel.to_owned(), command.to_owned());
            diesel::insert_into(silence::table)
                .values(DbSilence { channel: channel.to_owned(), command: command.to_owned() })
                .execute(&self.conn)?;
        }
        Ok(())
    }
}

fn establish_connection() -> PgConnection {
    let database_url = from_env("DATABASE_URL");
    PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

fn load_properties(conn: &PgConnection) -> HashMap<String, String> {
    use super::schema::property;
    HashMap::from_iter(
        property::table.load(conn)
            .expect("Error loading properties")
            .into_iter()
            .map(|x: Property| (x.key, x.value))
    )
}

fn load_reminders(conn: &PgConnection) -> MultiMap<String, Reminder> {
    use super::schema::reminder;
    MultiMap::from_iter(
        reminder::table.load(conn)
            .expect("Error loading reminders")
            .into_iter()
            .map(|x: Reminder| (x.nick.to_owned(), x))
    )
}

fn load_silences(conn: &PgConnection) -> MultiMap<String, String> {
    use super::schema::silence;
    MultiMap::from_iter(
        silence::table.load(conn)
            .expect("Error loading reminders")
            .into_iter()
            .map(|x: Silence| (x.channel, x.command))
    )
}

fn load_users(conn: &PgConnection) -> HashMap<String, User> {
    use super::schema::user;
    HashMap::from_iter(
        user::table.load(conn)
            .expect("Error loading users")
            .into_iter()
            .map(|x: User| (x.nick.to_owned(), x))
    )   
}
