use diesel::prelude::*;
use diesel::pg::PgConnection;
use hashbrown::HashMap;
use multimap::MultiMap;
use reqwest::Client;
use std::borrow::ToOwned;
#[cfg(not(test))] use std::iter::*;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::TryRecvError::{Empty, Disconnected};
use std::time::SystemTime;


#[macro_use] mod model_macro;
mod ban;
pub mod model;
pub mod schema;

use crate::logging;
use crate::local::LocalMap;
use crate::{Context, env, util};
use self::ban::Bans;

pub use self::model::*;
#[cfg(not(test))] use self::schema::*;

pub fn log<T>(res: QueryResult<T>) {
    if let Err(e) = res {
        logging::log(logging::WARN, &format!("DB error: {}", e));
    }
}

pub struct Db {
    #[cfg(not(test))] 
    pub conn: PgConnection,
    #[cfg(test)]
    pub seen: LocalMap<Seen>,

    pub client:   Client,
    pub nick:     String,
    pub owner:    Option<String>,
    pub owner_:   Option<String>,
    pub bans:     Option<Bans>,
    pub choices:  Vec<String>,

    pub reminders: MultiMap<String, Reminder>,
    pub silences:  LocalMap<Silence>,
    pub tells:     MultiMap<String, Tell>,
    pub users:     HashMap<String, User>,

    pub titles:    HashMap<String, String>,
    pub titles_r:  Option<Receiver<(String, String)>>
}

impl Db {
    pub fn new() -> Self {
        Self::establish_db().expect("Error loading database")
    }

    #[cfg(not(test))]
    fn establish_db() -> QueryResult<Self> {
        let conn = establish_connection();
        let owner = env::opt("OWNER");
        Ok(Db {
            client:    Client::new(),
            nick:      env::get("IRC_NICK").to_lowercase(),
            owner_:    env::opt("OWNER").map(|x| x.to_lowercase()),
            owner,
            bans:      Bans::new(),
            choices:   Vec::new(),
            reminders: load_reminders(&conn)?,
            silences:  load_silences(&conn)?,
            tells:     load_tells(&conn)?,
            users:     load_users(&conn)?,

            titles:    HashMap::new(),
            titles_r:  None,
            conn
        })
    }
    #[cfg(test)]
    fn establish_db() -> QueryResult<Self> {
        let owner = env::opt("OWNER");
        env::load();
        Ok(Db {
            client:    Client::new(),
            nick:      env::get("IRC_NICK").to_lowercase(),
            owner_:    env::opt("OWNER").map(|x| x.to_lowercase()),
            owner,
            bans:      Bans::new(),
            choices:   Vec::new(),
            reminders: MultiMap::new(),
            silences:  LocalMap::new(),
            tells:     MultiMap::new(),
            users:     HashMap::new(),
            seen:      LocalMap::new(),

            titles:    HashMap::new(),
            titles_r:  None
        })
    }

    pub fn listen(&mut self) {
        if let Some(titles_r) = &self.titles_r {
            loop {
                match titles_r.try_recv() {
                    Err(Empty)        => break,
                    Err(Disconnected) => { self.titles_r = None; break },
                    Ok((k, v))        => {
                        if v == "[ACCESS DENIED]" {
                            self.titles.remove(&k);
                        } else {
                            self.titles.insert(k, v);
                        }
                    }
                }
            }
        }
    }

    #[cfg(not(test))]
    pub fn reload(&mut self) -> QueryResult<()> {
        self.reminders = load_reminders(&self.conn)?;
        self.silences  = load_silences(&self.conn)?;
        self.tells     = load_tells(&self.conn)?;
        self.users     = load_users(&self.conn)?;
        self.bans      = Bans::new();
        Ok(())
    }
    #[cfg(test)]
    pub fn reload(&mut self) -> QueryResult<()> {
        Ok(())
    }


    pub fn auth(&self, nick: &str) -> i32 {
        let user = nick.to_lowercase();
        match &self.owner_ {
            _ if self.nick == user        => 5,
            Some(owner) if owner == &user => 4,
            _ => match self.users.get(&user) {
                None => 0,
                Some(u) => u.auth
            }
        }
    }

    pub fn get_reminders(&mut self, ctx: &Context) -> Option<Vec<Reminder>> {
        let when = SystemTime::now();
        let mut reminders = self.reminders.get_vec_mut(&ctx.user)?;
        let expired = util::drain_filter(&mut reminders, |x| x.when < when);
        
        #[cfg(not(test))] {
        if !expired.is_empty() {
                log(diesel
                    ::delete(reminder::table
                        .filter(reminder::user.eq(&ctx.user))
                        .filter(reminder::when.lt(&when))
                    ).execute(&self.conn));
            }
        }
        
        Some(expired)
    }

    pub fn get_tells(&mut self, ctx: &Context) -> Option<Vec<Tell>> {
        let tells = self.tells.remove(&ctx.user)?;
        
        #[cfg(not(test))] {
        if !tells.is_empty() {
            log(diesel
                ::delete(tell::table.filter(tell::target.eq(&ctx.user)))
                .execute(&self.conn));
            }
        }
        
        Some(tells)
    }


    pub fn add_seen(&mut self, ctx: &Context, message: &str) -> QueryResult<()> {
        if ctx.channel != ctx.user && ctx.user != self.nick {
            let when = SystemTime::now();
            let seen = Seen {
                channel: ctx.channel.to_owned(),
                user:    ctx.user.to_owned(),
                first:   message.to_owned(), first_time:  when, 
                latest:  message.to_owned(), latest_time: when,
                total:   1 
            };
            #[cfg(not(test))] diesel
                ::insert_into(seen::table)
                .values(&seen)
                .on_conflict((seen::channel, seen::user))
                .do_update()
                .set((
                    seen::latest.eq(message),
                    seen::latest_time.eq(&when),
                    seen::total.eq(seen::total + 1)
                )).execute(&self.conn)?;
            #[cfg(test)] {
                if self.replace_seen(&seen).is_none() {
                    self.seen.insert(seen);
                }
            }
        }
        Ok(())
    }
    #[cfg(test)]
    fn replace_seen(&mut self, seen: &Seen) -> Option<()> {
        let old = self.seen.remove_by(seen)?;
        self.seen.insert(Seen { 
            latest:      seen.latest.to_owned(),
            latest_time: seen.latest_time, 
            total:       old.total + 1,
            ..old
        });
        Some(())
    }

    #[cfg(not(test))]
    pub fn get_seen(&self, channel: &str, nick: &str) -> QueryResult<Seen> {
        seen::table
            .filter(seen::channel.eq(&channel.to_lowercase()))
            .filter(seen::user.eq(&nick.to_lowercase()))
            .first::<DbSeen>(&self.conn)
            .map(Seen::from)
    }
    #[cfg(test)]
    pub fn get_seen(&self, channel: &str, nick: &str) -> QueryResult<Seen> {
        let res = self.seen.get(&channel.to_lowercase(), &nick.to_lowercase())
            .ok_or(diesel::result::Error::NotFound)?;
        Ok(res.to_owned())
    }
}

pub fn establish_connection() -> PgConnection {
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
            .map(|x: DbReminder| (x.user.to_owned(), Reminder::from(x)))
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
fn load_silences(conn: &PgConnection) -> QueryResult<LocalMap<Silence>> {
    Ok(LocalMap::from_iter(
        silence::table.load::<DbSilence>(conn)?
            .into_iter()
            .map(Silence::from)
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
