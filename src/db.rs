use diesel::prelude::*;
use diesel::pg::PgConnection;
use multimap::MultiMap;
use reqwest::Client;
use std::collections::HashMap;
use std::iter::*;
use std::time::SystemTime;

mod ban;
#[macro_use] mod model_macro;
mod model;
mod name;
mod schema;

use crate::{Api, color, env, util};
use crate::response::choice::Choices;
use crate::response::wikidot::Wikidot;
use self::ban::Bans;
use self::model::*;
#[cfg(not(test))] use self::schema::*;
use self::name::Names;

pub fn log<T>(res: QueryResult<T>) {
    if let Err(e) = res {
        color::log(color::WARN, &format!("DB error: {}", e));
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Apis {
    pub google:  Option<Api>,
    pub wikidot: Option<Wikidot>
}

impl Apis {
    pub fn new() -> Self {
        #[cfg(test)] env::load();
        Apis {
            google:  env::api("GOOGLE", "CUSTOMENGINE", "KEY"),
            wikidot: Wikidot::new()
        }
    }
}

pub struct Db {
    #[cfg(not(test))] 
    conn: PgConnection,
    #[cfg(test)]
    seen: MultiMap<String, Seen>,

    pub client:  Client,
    pub nick:    String,
    pub owner:   Option<String>,
    pub api:     Apis,
    pub bans:    Option<Bans>,
    pub choices: Choices,
    pub names:   Names,

    reminders:   MultiMap<String, Reminder>,
    silences:    MultiMap<String, String>,
    tells:       MultiMap<String, Tell>,
    users:       HashMap<String, User>
}

impl Db {
    pub fn new() -> Self {
        Self::establish_db().expect("Error loading database")
    }

    #[cfg(not(test))]
    fn establish_db() -> QueryResult<Self> {
        let conn = establish_connection();
        Ok(Db {
            client:     Client::new(),
            nick:       env::get("IRC_NICK").to_lowercase(),
            owner:      env::opt("OWNER").map(|x| x.to_lowercase()),
            api:        Apis::new(),
            bans:       Bans::new(),
            choices:    Choices::new(),
            names:      Names::new(&conn).expect("Error loading names"),
            reminders:  load_reminders(&conn)?,
            silences:   load_silences(&conn)?,
            tells:      load_tells(&conn)?,
            users:      load_users(&conn)?,
            conn
        })
    }
    #[cfg(test)]
    fn establish_db() -> QueryResult<Self> {
        env::load();
        Ok(Db {
            client:     Client::new(),
            nick:       env::get("IRC_NICK").to_lowercase(),
            owner:      env::opt("OWNER").map(|x| x.to_lowercase()),
            api:        Apis::new(),
            bans:       Bans::new(),
            choices:    Choices::new(),
            names:      Names::empty(),
            reminders:  MultiMap::new(),
            silences:   MultiMap::new(),
            tells:      MultiMap::new(),
            users:      HashMap::new(),
            seen:       MultiMap::new()
        })
    }

    #[cfg(not(test))]
    pub fn reload(&mut self) -> QueryResult<()> {
        self.reminders = load_reminders(&self.conn)?;
        self.silences  = load_silences(&self.conn)?;
        self.users     = load_users(&self.conn)?;
        self.names     = Names::new(&self.conn).expect("Error loading names");
        self.bans      = Bans::new();
        Ok(())
    }
    #[cfg(test)]
    pub fn reload(&mut self) -> QueryResult<()> {
        Ok(())
    }


    fn get_auth(&self, nick_up: &str) -> i32 {
        let nick = nick_up.to_lowercase();
        if self.nick == nick {
            5
        } else if self.owner == Some(nick.to_owned()) {
            4
        } else { 
            self.users.get(&nick).map(|x| x.auth).unwrap_or(0) 
        }
    }

    pub fn auth(&self, level: i32, user: &str) -> bool {
        level <= self.get_auth(user)
    }

    pub fn outranks(&self, x: &str, y: &str) -> bool {
        self.get_auth(x) > self.get_auth(y)
    }


    pub fn add_user(&mut self, auth: i32, nick_up: &str) -> QueryResult<()> {
        let nick = nick_up.to_lowercase();
        let user = User {
            nick: nick.to_owned(),
            auth,
            pronouns: self.users.get(&nick).and_then(|x| x.pronouns.to_owned())
        };
        #[cfg(not(test))] diesel
            ::insert_into(user::table)
            .values(&user)
            .on_conflict(user::nick)
            .do_update()
            .set(user::auth.eq(auth))
            .execute(&self.conn)?;
        self.users.insert(nick, user);
        Ok(())
    }

    pub fn delete_user(&mut self, nick_up: &str) -> QueryResult<bool> {
        let nick = nick_up.to_lowercase();
        let removed = self.users.remove(&nick);
        #[cfg(not(test))] diesel
            ::delete(user::table.filter(user::nick.eq(&nick)))
            .execute(&self.conn)?;
        #[cfg(not(test))] diesel
            ::delete(seen::table.filter(seen::nick.eq(&nick)))
            .execute(&self.conn)?;
        Ok(removed.is_some())
    }

    
    pub fn add_reminder(&mut self, nick_up: &str, when: SystemTime, message: &str) 
    -> QueryResult<()> {
        let nick = nick_up.to_lowercase();
        let reminder = Reminder {
            nick:    nick.to_owned(),
            when:    when,
            message: message.to_owned()
        };
        #[cfg(not(test))] diesel
            ::insert_into(reminder::table)
            .values(&reminder)
            .execute(&self.conn)?;
        self.reminders.insert(nick, reminder);
        Ok(())
    }
    pub fn get_reminders(&mut self, nick_up: &str) -> Option<Vec<Reminder>> {
        let nick = nick_up.to_lowercase();
        let when = SystemTime::now();
        let mut reminders = self.reminders.get_vec_mut(&nick)?;
        let expired = util::drain_filter(&mut reminders, |x| x.when < when);
        #[cfg(not(test))] log(diesel
            ::delete(reminder::table
                .filter(reminder::nick.eq(&nick))
                .filter(reminder::when.lt(&when))
            ).execute(&self.conn));
        Some(expired)
    }


    pub fn add_tell(&mut self, sender: &str, target_up: &str, message: &str) -> QueryResult<()> {
        let target = target_up.to_lowercase();
        let tell = Tell {
            sender:  sender.to_owned(),
            target:  target.to_owned(),
            time:    SystemTime::now(),
            message: message.to_owned()
        };
        #[cfg(not(test))] diesel
            ::insert_into(tell::table)
            .values(&tell)
            .execute(&self.conn)?;
        self.tells.insert(target, tell);
        Ok(())
    }

    pub fn get_tells(&mut self, nick_up: &str) -> Option<Vec<Tell>> {
        let nick = nick_up.to_lowercase();
        let tells = self.tells.remove(&nick)?;
        #[cfg(not(test))] log(diesel
            ::delete(tell::table.filter(tell::target.eq(&nick)))
            .execute(&self.conn));
        Some(tells)
    }


    pub fn silenced(&self, channel_up: &str, command: &str) -> bool {
        match self.silences.get_vec(&channel_up.to_lowercase()) {
            None => false,
            Some(silence) => silence.contains(&command.to_owned())
        }
    }

    pub fn set_enabled(&mut self, channel_up: &str, command_up: &str, enabled: bool) 
    -> QueryResult<()> {
        let channel = channel_up.to_lowercase();
        let command = command_up.to_lowercase();
        if enabled {
            util::multi_remove(&mut self.silences, &channel, &command);
            #[cfg(not(test))] diesel
                ::delete(silence::table
                .filter(silence::channel.eq(&channel))
                .filter(silence::command.eq(&command))
            ).execute(&self.conn)?;
        } else {
            self.silences.insert(channel.to_owned(), command.to_owned());
            #[cfg(not(test))] diesel
                ::insert_into(silence::table)
                .values(&Silence { channel, command })
                .execute(&self.conn)?;
        }
        Ok(())
    }


    pub fn add_seen(&mut self, channel_up: &str, nick_up: &str, message: &str) -> QueryResult<()> {
        let channel = channel_up.to_lowercase();
        let nick = nick_up.to_lowercase();
        if channel != nick && channel != self.nick {
            let when = SystemTime::now();
            let seen = Seen {
                channel: channel,
                nick:    nick,
                first:   message.to_owned(), first_time:  when, 
                latest:  message.to_owned(), latest_time: when,
                total:   1 
            };
            #[cfg(not(test))] diesel
                ::insert_into(seen::table)
                .values(&seen)
                .on_conflict((seen::channel, seen::nick))
                .do_update()
                .set((
                    seen::latest.eq(message),
                    seen::latest_time.eq(&when),
                    seen::total.eq(seen::total + 1)
                )).execute(&self.conn)?;
            #[cfg(test)]
            match self.replace_seen(&channel_up.to_lowercase(), &seen) {
                None => self.seen.insert(channel_up.to_lowercase(), seen),
                Some(_) => ()
            }
        }
        Ok(())
    }
    #[cfg(test)]
    fn replace_seen(&mut self, channel: &str, seen: &Seen) -> Option<()> {
        let chan = self.seen.get_vec_mut(channel)?;
        let old = util::drain_filter(chan, |x| x.nick == seen.nick).first()?.to_owned();
        chan.push(Seen { 
            latest:      seen.latest.to_owned(),
            latest_time: seen.latest_time, 
            total:       old.total + 1,
            ..old
        });
        Some(())
    }

    #[cfg(not(test))]
    pub fn get_seen(&self, channel_up: &str, nick_up: &str) -> QueryResult<Option<Seen>> {
        Ok(seen::table
            .filter(seen::channel.eq(&channel_up.to_lowercase()))
            .filter(seen::nick.eq(&nick_up.to_lowercase()))
            .first::<DbSeen>(&self.conn)
            .optional()?
            .map(Seen::from)
        )
    }
    #[cfg(test)]
    pub fn get_seen(&self, channel_up: &str, nick_up: &str) -> QueryResult<Option<Seen>> {
        Ok(self.load_seen(channel_up, nick_up))
    }
    #[cfg(test)]
    fn load_seen(&self, channel_up: &str, nick_up: &str) -> Option<Seen> {
        let channel = channel_up.to_lowercase();
        let nick = nick_up.to_lowercase();
        let el = self.seen.get_vec(&channel)?.into_iter().find(|x| x.nick == nick)?;
        Some(el.to_owned())
    }
}

fn establish_connection() -> PgConnection {
    #[cfg(test)] env::load();
    let database_url = env::get("DATABASE_URL");
    PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

#[cfg(not(test))]
fn load_reminders(conn: &PgConnection) -> QueryResult<MultiMap<String, Reminder>> {
    Ok(MultiMap::from_iter(
        reminder::table.load(conn)?
            .into_iter()
            .map(|x: DbReminder| (x.nick.to_owned(), Reminder::from(x)))
    ))
}

#[cfg(not(test))]
fn load_tells(conn: &PgConnection) -> QueryResult<MultiMap<String, Tell>> {
    Ok(MultiMap::from_iter(
        tell::table.load(conn)?
            .into_iter()
            .map(|x: DbTell| (x.target.to_owned(), Tell::from(x)))
    ))
}

#[cfg(not(test))]
fn load_silences(conn: &PgConnection) -> QueryResult<MultiMap<String, String>> {
    Ok(MultiMap::from_iter(
        silence::table.load(conn)?
            .into_iter()
            .map(|x: DbSilence| (x.channel, x.command))
    ))
}

#[cfg(not(test))]
fn load_users(conn: &PgConnection) -> QueryResult<HashMap<String, User>> {
    Ok(HashMap::from_iter(
        user::table.load(conn)?
            .into_iter()
            .map(|x: User| (x.nick.to_owned(), x))
    ))
}
