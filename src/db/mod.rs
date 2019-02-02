use diesel::prelude::*;
use diesel::query_dsl::RunQueryDsl;
use diesel::pg::PgConnection;
use diesel::pg::upsert::excluded;
use diesel::r2d2::ConnectionManager;
use hashbrown::HashMap;
use multimap::MultiMap;
use r2d2::PooledConnection;
use reqwest::Client;
use std::borrow::ToOwned;
use std::iter::*;
use std::sync::mpsc::TryRecvError::{Empty, Disconnected};
use std::time::SystemTime;

#[macro_use] mod model_macro;
mod ban;
mod model;
pub mod pages;
mod schema;

use crate::background::diff::DiffReceiver;
use crate::logging::*;
use crate::local::LocalMap;
use crate::{Context, env, util};
use crate::output::Output;
use crate::wikidot::Wikidot;
use self::ban::Bans;

pub use self::model::*;
pub use self::schema::*;

pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;
pub type Conn = PooledConnection<ConnectionManager<PgConnection>>;

pub struct Db {
    nick:      String,
    pub owner: String,
    owner_:    String,

    pub bans:      Option<Bans>,
    pub choices:   Vec<String>,
    pub reminders: MultiMap<String, Reminder>,
    pub silences:  LocalMap<Silence>,
    pub tells:     MultiMap<String, Tell>,
    pub wiki:      Wikidot,

    pub titles:    HashMap<String, String>,
    pub titles_r:  Option<DiffReceiver<(String, String)>>,

    pub client:   Client,
    pool:         Pool
}

impl Default for Db { 
    fn default() -> Self { 
        #[cfg(test)] env::load();
        Self::new(establish_connection())
    }
}

impl Db {
    pub fn new(pool: Pool) -> Self {
        let owner = env::get("OWNER");
        let mut db = Db {
            client:    Client::new(),
            nick:      env::get("IRC_NICK").to_lowercase(),
            owner_:    owner.to_lowercase(),
            owner,
            bans:      None,
            choices:   Vec::new(),
            reminders: MultiMap::new(),
            silences:  LocalMap::new(),
            tells:     MultiMap::new(),
            wiki:      Wikidot::new(),

            titles:    HashMap::new(),
            titles_r:  None,

            pool
        };
        db.reload().expect("Error loading database");
        db
    }

    pub fn conn(&self) -> Conn {
        self.pool.get().expect("Failed to get connection from database pool")
    }

    pub fn title(&self, page: &Page) -> String {
        match self.titles.get(&page.id) {
            None        => page.title.to_owned(),
            Some(title) => format!("{}: {}", page.title, title)
        }
    }

    pub fn listen(&mut self) {
        if let Some(titles_r) = &self.titles_r {
            loop {
                match titles_r.try_recv() {
                    Err(Empty)          => break,
                    Err(Disconnected)   => { self.titles_r = None; break },
                    Ok(((k, _), false)) => { self.titles.remove(&k); },
                    Ok(((k, v), true))  => {
                        let title = format!("{}: {}", k.to_uppercase(), v);
                        diesel::update
                            (page::table.filter(page::id.eq(&k)).filter(page::title.ne(&title)))
                            .set(page::title.eq(&title))
                            .execute(&self.conn())
                            .log(trace!());
                        self.titles.insert(k, v);
                    }
                }
            }
        }
    }
    
    #[cfg(not(test))]
    fn retrieve<Frm, To, C, L, F>(&self, table: L, f: F) -> QueryResult<C>
    where C: FromIterator<To>, L: diesel::query_dsl::LoadQuery<PgConnection, Frm>, F: Fn(Frm) -> To {
        Ok(table.load::<Frm>(&self.conn())?.into_iter().map::<To, F>(f).collect())
    }


    #[cfg(not(test))]
    pub fn reload(&mut self) -> QueryResult<()> {
        let conn = self.conn();
        #[cfg(not(test))] { self.bans = Bans::build(); }
        self.silences = silence::table.load(&conn)?.into_iter().collect();
        self.reminders = self.retrieve::<DbReminder,_,_,_,_>
            (reminder::table, |x| (x.user.to_owned(), Reminder::from(x)))?;
        self.tells = self.retrieve::<DbTell,_,_,_,_>
            (tell::table, |x| (x.target.to_owned(), Tell::from(x)))?;
        Ok(())
    }
    #[cfg(test)]
    pub fn reload(&mut self) -> QueryResult<()> {
        self.owner_ = self.owner.to_lowercase();
        Ok(())
    }

    pub fn auth<T: Output>(&self, ctx: &Context, irc: &T) -> u8 {
        if ctx.user == self.nick {
            5
        } else if ctx.user == self.owner_ {
            4
        } else {
            irc.auth(ctx)
        }
    }

    pub fn get_reminders(&mut self, ctx: &Context) -> Option<Vec<Reminder>> {
        let when = SystemTime::now();
        let mut reminders = self.reminders.get_vec_mut(&ctx.user)?;
        let expired = util::drain_filter(&mut reminders, |x| x.when < when);
        
        if !expired.is_empty() {
            diesel::delete(
                reminder::table
                    .filter(reminder::user.eq(&ctx.user))
                    .filter(reminder::when.lt(&when))
                )
                .execute(&self.conn())
                .log(trace!());
        }
        
        Some(expired)
    }

    pub fn get_tells(&mut self, ctx: &Context) -> Option<Vec<Tell>> {
        let tells = self.tells.remove(&ctx.user)?;
        
        if !tells.is_empty() {
            diesel::delete(tell::table.filter(tell::target.eq(&ctx.user)))
            .execute(&self.conn())
            .log(trace!());
        }
        
        Some(tells)
    }


    pub fn add_seen(&mut self, ctx: &Context, message: &str) -> QueryResult<()> {
        if ctx.channel != ctx.user && ctx.user != self.nick {
            let seen = SeenInsert {
                channel: ctx.channel.to_owned(),
                user:    ctx.user.to_owned(),
                first:   message.to_owned(),
                latest:  message.to_owned()
            };
            diesel::insert_into(seen::table)
                .values(&seen)
                .on_conflict((seen::channel, seen::user))
                .do_update()
                .set((
                    seen::latest.eq(excluded(seen::latest)),
                    seen::latest_time.eq(excluded(seen::latest_time)),
                    seen::total.eq(seen::total + 1)
                ))
            .execute(&self.conn())?;
        }
        Ok(())
    }

    pub fn get_seen(&self, channel: &str, nick: &str) -> QueryResult<Seen> {
        seen::table
            .filter(seen::channel.eq(&channel.to_lowercase()))
            .filter(seen::user.eq(&nick.to_lowercase()))
        .first(&self.conn())
    }
}

#[cfg(not(test))]
pub fn establish_connection() -> Pool {
    r2d2::Pool::builder()
        .max_size(env::get("DATABASE_POOL").parse().expect("Invalid DATABASE_POOL number"))
        .build(ConnectionManager::new(env::get("DATABASE_URL")))
        .expect("Error connecting to database")
}

#[cfg(test)]
pub fn establish_connection() -> Pool {
    r2d2::Pool::builder()
        .max_size(1)
        .connection_timeout(std::time::Duration::new(0, 1))
        .build_unchecked(ConnectionManager::new(""))
}
