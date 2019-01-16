use diesel::prelude::*;
use diesel::pg::PgConnection;
#[cfg(not(test))] use diesel::query_dsl::methods::LoadQuery;
use hashbrown::{HashSet, HashMap};
use multimap::MultiMap;
use reqwest::Client;
use std::borrow::ToOwned;
#[cfg(not(test))] use std::iter::*;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::TryRecvError::{Empty, Disconnected};
use std::time::SystemTime;

#[macro_use] mod model_macro;
mod ban;
mod model;
mod schema;

use crate::logging;
use crate::local::LocalMap;
use crate::{Context, IO, env, util};
use crate::wikidot::Wikidot;
use self::ban::Bans;

pub use self::model::*;
pub use self::schema::*;

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

    pub loaded:    HashSet<String>,
    pub loaded_r:  Option<Receiver<(String, bool)>>,
    pub titles:    HashMap<String, String>,
    pub titles_r:  Option<Receiver<(String, String)>>,

    pub wiki:      Option<Wikidot>
}

impl Db {
    pub fn new() -> Self {
        let mut db = Self::establish_db();
        db.reload().expect("Error loading database");
        db
    }

    fn establish_db() -> Self {
        let owner = env::opt("OWNER");
        env::load();
        Db {
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

            loaded:    HashSet::new(),
            titles:    HashMap::new(),
            loaded_r:  None,
            titles_r:  None,
            wiki:      Wikidot::new(),

            #[cfg(not(test))]
            conn:      establish_connection(),
            #[cfg(test)]
            seen:      LocalMap::new()
        }
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
        if let Some(loaded_r) = &self.loaded_r {
            let mut added = Vec::new();
            let mut deleted = Vec::new();
            loop {
                match loaded_r.try_recv() {
                    Err(Empty)        => break,
                    Err(Disconnected) => { self.titles_r = None; break },
                    Ok((k, true))     => added.push(k),
                    Ok((k, false))    => deleted.push(k)
                }
            }
            
        }
    }

    #[cfg(not(test))]
    fn load<Frm, To, C, L, F>(&self, table: L, f: F) -> QueryResult<C>
    where C: FromIterator<To>, L: LoadQuery<PgConnection, Frm>, F: Fn(Frm) -> To {
        Ok(C::from_iter(table.load::<Frm>(&self.conn)?.into_iter().map::<To, F>(f)))
    }

    #[cfg(not(test))]
    pub fn reload(&mut self) -> QueryResult<()> {
        self.bans = Bans::new();
        self.loaded = HashSet::from_iter(
            page::table.select(page::fullname).get_results(&self.conn)?.into_iter()
        );
        self.reminders = 
            self.load(reminder::table, |x: DbReminder| (x.user.to_owned(), Reminder::from(x)))?;
        self.silences = 
            self.load(silence::table,  |x: DbSilence|  Silence::from(x))?;
        self.tells = 
            self.load(tell::table,     |x: DbTell|     (x.target.to_owned(), Tell::from(x)))?;
        self.users = 
            self.load(user::table,     |x: User|       (x.nick.to_owned(), x))?;
        Ok(())
    }
    #[cfg(test)]
    pub fn reload(&mut self) -> QueryResult<()> {
        Ok(())
    }

    pub fn with_title(&self, s: &str) -> String {
        match self.titles.get(s) {
            None => s.to_owned(),
            Some(title) => {
                if s.starts_with("scp-") {
                  format!("{}: {}", s.to_uppercase(), title)
                } else {
                    title.to_owned()
                }
            }
        }
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

    #[cfg(not(test))]
    fn download_diff(&mut self, added: Vec<String>, deleted: Vec<String>) -> IO<()> {
        if let Some(wiki) = &self.wiki {
            for x in deleted {
                diesel
                    ::delete(page::table.filter(page::fullname.eq(&x)))
                    .execute(&self.conn)?;
                diesel
                    ::delete(tag::table.filter(tag::page.eq(&x)))
                    .execute(&self.conn)?;
                self.loaded.remove(&x);
            }
            wiki.walk(&added, &self.client, |title, mut page, tags| {
                if let Some(title) = self.titles.get(&page.fullname) {
                    page.title.push_str(": ");
                    page.title.push_str(title);
                }
                diesel
                    ::insert_into(page::table)
                    .values(&page)
                    .on_conflict_do_nothing()
                    .execute(&self.conn)?;
                for tag in tags {
                    diesel
                        ::insert_into(tag::table)
                        .values(Tag { name: tag, page: title.to_owned() })
                        .execute(&self.conn)?;
                }
                Ok(())
            })?;
            for x in added {
                self.loaded.insert(x);
            }
        }
        Ok(())
    }
    #[cfg(test)]
    fn download_diff(&mut self, _: Vec<String>, _: Vec<String>) -> IO<()> {
        Ok(())
    }


    pub fn download(&mut self, titles: &HashSet<String>) -> IO<()> {
        let added = titles
            .difference(&self.loaded)
            .map(ToOwned::to_owned)
            .collect();
        let deleted = self.loaded
            .difference(&titles)
            .map(ToOwned::to_owned)
            .collect();
        self.download_diff(added, deleted)
    }
}

pub fn establish_connection() -> PgConnection {
    #[cfg(test)] env::load();
    let database_url = env::get("DATABASE_URL");
    PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}
