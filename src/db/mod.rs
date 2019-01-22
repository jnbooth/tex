use diesel::prelude::*;
use diesel::helper_types::Limit;
use diesel::query_dsl::methods::{ExecuteDsl, LimitDsl};
use diesel::query_dsl::{LoadQuery, RunQueryDsl};
use diesel::pg::PgConnection;
use hashbrown::{HashSet, HashMap};
use multimap::MultiMap;
use reqwest::Client;
use std::borrow::ToOwned;
use std::iter::*;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::TryRecvError::{Empty, Disconnected};
use std::time::SystemTime;

#[macro_use] mod model_macro;
mod ban;
mod model;
pub mod pages;
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
    nick:      String,
    pub owner: Option<String>,
    owner_:    Option<String>,

    pub bans:      Option<Bans>,
    pub choices:   Vec<String>,
    pub reminders: MultiMap<String, Reminder>,
    pub silences:  LocalMap<Silence>,
    pub tells:     MultiMap<String, Tell>,
    pub users:     HashMap<String, User>,
    pub wiki:      Option<Wikidot>,

    pub loaded:    HashSet<String>,
    pub loaded_r:  Option<Receiver<(String, bool)>>,
    pub titles:    HashMap<String, String>,
    pub titles_r:  Option<Receiver<(String, String)>>,

    pub client:   Client,
    #[cfg(not(test))] 
    conn: PgConnection,
    #[cfg(test)]
    seen: LocalMap<Seen>
}

impl Default for Db { 
    fn default() -> Self { 
        #[cfg(test)] env::load();
        let owner = env::opt("OWNER");
        Db {
            client:    Client::new(),
            nick:      env::get("IRC_NICK").to_lowercase(),
            owner_:    env::opt("OWNER").map(|x| x.to_lowercase()),
            owner,
            bans:      None,
            choices:   Vec::new(),
            reminders: MultiMap::new(),
            silences:  LocalMap::new(),
            tells:     MultiMap::new(),
            users:     HashMap::new(),
            wiki:      Wikidot::build(),

            loaded:    HashSet::new(),
            titles:    HashMap::new(),
            loaded_r:  None,
            titles_r:  None,

            #[cfg(not(test))]
            conn:      establish_connection(),
            #[cfg(test)]
            seen:      LocalMap::new()
        }
    } 
}

impl Db {
    pub fn new() -> Self {
        let mut db = Self::default();
        db.reload().expect("Error loading database");
        db
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
                            let title = format!("{}: {}", k.to_uppercase(), v);
                            log(self.execute(diesel
                                ::update(page::table
                                    .filter(page::fullname.eq(&k))
                                    .filter(page::title.ne(&title))
                                ).set(page::title.eq(&title))));
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

    pub fn reload(&mut self) -> QueryResult<()> {
        #[cfg(not(test))] { self.bans = Bans::build(); }
        self.loaded = self.load(page::table.select(page::fullname))?.into_iter().collect();
        self.silences = self.load(silence::table)?.into_iter().collect();
        self.reminders = self.retrieve::<DbReminder,_,_,_,_>
            (reminder::table, |x| (x.user.to_owned(), Reminder::from(x)))?;
        self.tells = self.retrieve::<DbTell,_,_,_,_>
            (tell::table, |x| (x.target.to_owned(), Tell::from(x)))?;
        self.users = self.retrieve::<User,_,_,_,_>
            (user::table, |x| (x.nick.to_owned(), x))?;
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
        
        if !expired.is_empty() {
            log(self.execute(diesel
                ::delete(reminder::table
                    .filter(reminder::user.eq(&ctx.user))
                    .filter(reminder::when.lt(&when))
            )));
        }
        
        Some(expired)
    }

    pub fn get_tells(&mut self, ctx: &Context) -> Option<Vec<Tell>> {
        let tells = self.tells.remove(&ctx.user)?;
        
        if !tells.is_empty() {
            log(self.execute(diesel::delete(tell::table.filter(tell::target.eq(&ctx.user)))));
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
            self.execute(diesel
                ::insert_into(seen::table)
                .values(&seen)
                .on_conflict((seen::channel, seen::user))
                .do_update()
                .set((
                    seen::latest.eq(message),
                    seen::latest_time.eq(&when),
                    seen::total.eq(seen::total + 1)
                ))
            )?;
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
        self.first(seen::table
            .filter(seen::channel.eq(&channel.to_lowercase()))
            .filter(seen::user.eq(&nick.to_lowercase()))
        )
    }
    #[cfg(test)]
    pub fn get_seen(&self, channel: &str, nick: &str) -> QueryResult<Seen> {
        let res = self.seen.get(&channel.to_lowercase(), &nick.to_lowercase())
            .ok_or(diesel::result::Error::NotFound)?;
        Ok(res.to_owned())
    }

    fn download_diff(&mut self, added: Vec<String>, deleted: Vec<String>) -> IO<()> {
        if let Some(wiki) = &self.wiki {
            for x in deleted {
                self.execute(diesel::delete(page::table.filter(page::fullname.eq(&x))))?;
                self.execute(diesel::delete(tag::table.filter(tag::page.eq(&x))))?;
                self.loaded.remove(&x);
            }
            wiki.walk(&added, &self.client, |title, mut page, tags: Vec<String>| {
                if let Some(title) = self.titles.get(&page.fullname) {
                    page.title.push_str(": ");
                    page.title.push_str(title);
                }
                self.execute(diesel
                    ::insert_into(page::table)
                    .values(&page)
                    .on_conflict(page::fullname)
                    .do_update()
                    .set(page::rating.eq(page.rating))
                )?;
                for tag in tags {
                    self.execute(diesel
                        ::insert_into(tag::table)
                        .values(Tag { name: tag, page: title.to_owned() })
                        .on_conflict_do_nothing()
                    )?;
                }
                Ok(())
            })?;
            for x in added {
                self.loaded.insert(x);
            }
        }
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

    

    #[cfg(not(test))]
    pub fn execute<T> (&self, t: T) -> QueryResult<usize> 
    where T: RunQueryDsl<PgConnection> + ExecuteDsl<PgConnection> {
        t.execute(&self.conn)
    }
    #[cfg(test)]
    pub fn execute<T> (&self, _: T) -> QueryResult<usize> 
    where T: RunQueryDsl<PgConnection> + ExecuteDsl<PgConnection> {
        Ok(0)
    }
    #[cfg(not(test))]
    pub fn load<U, T>(&self, t: T) -> QueryResult<Vec<U>>
    where T: RunQueryDsl<PgConnection> + LoadQuery<PgConnection, U> {
        t.load(&self.conn)
    }
    #[cfg(test)]
    pub fn load<U, T>(&self, _: T) -> QueryResult<Vec<U>>
    where T: RunQueryDsl<PgConnection> + LoadQuery<PgConnection, U> {
        Ok(Vec::new())
    }
    #[cfg(not(test))]
    pub fn get_result<U, T>(&self, t: T) -> QueryResult<U>
    where T: RunQueryDsl<PgConnection> + LoadQuery<PgConnection, U> {
        t.get_result(&self.conn)
    }
    #[cfg(test)]
    pub fn get_result<U, T>(&self, _: T) -> QueryResult<U>
    where T: RunQueryDsl<PgConnection> + LoadQuery<PgConnection, U> {
        Err(diesel::result::Error::NotFound)
    }
    #[cfg(not(test))]
    pub fn first<U, T>(&self, t: T) -> QueryResult<U>
    where T: RunQueryDsl<PgConnection> + LimitDsl, Limit<T>: LoadQuery<PgConnection, U> {
        t.first(&self.conn)
    }
    #[cfg(test)]
    pub fn first<U, T>(&self, _: T) -> QueryResult<U>
    where T: RunQueryDsl<PgConnection> + LimitDsl, Limit<T>: LoadQuery<PgConnection, U> {
        Err(diesel::result::Error::NotFound)
    }
    #[cfg(not(test))]
    fn retrieve<Frm, To, C, L, F>(&self, table: L, f: F) -> QueryResult<C>
    where C: FromIterator<To>, L: LoadQuery<PgConnection, Frm>, F: Fn(Frm) -> To {
        Ok(table.load::<Frm>(&self.conn)?.into_iter().map::<To, F>(f).collect())
    }
    #[cfg(test)]
    fn retrieve<Frm, To, C, L, F>(&self, _: L, _: F) -> QueryResult<C>
    where C: FromIterator<To>, L: LoadQuery<PgConnection, Frm>, F: Fn(Frm) -> To {
        Ok(std::iter::empty().collect())
    }
}

pub fn establish_connection() -> PgConnection {
    let database_url = env::get("DATABASE_URL");
    PgConnection::establish(&database_url).expect("Error connecting to database")
}
