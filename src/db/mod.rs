use diesel::{Column, ExpressionMethods};
use diesel::pg::{Pg, PgConnection};
use diesel::pg::upsert::excluded;
use diesel::prelude::*;
use diesel::query_builder::{AsChangeset, QueryFragment};
use diesel::query_dsl::RunQueryDsl;
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
mod model;
pub mod pages;
mod schema;

use crate::{Context, IO, env, util};
use crate::logging::*;
use crate::local::LocalMap;
use crate::output::Output;
use crate::wikidot::Wikidot;
use crate::background::{Ban, DiffReceiver};

pub use self::model::*;
pub use self::schema::*;

pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;
pub type Conn = PooledConnection<ConnectionManager<PgConnection>>;

pub struct Db {
    nick:      String,
    pub owner: String,
    owner_:    String,

    pub choices:   Vec<String>,
    pub reminders: MultiMap<String, Reminder>,
    pub silences:  LocalMap<Silence>,
    pub tells:     MultiMap<String, Tell>,
    pub wiki:      Wikidot,

    pub bans:      MultiMap<String, Ban>,
    pub bans_r:    Option<DiffReceiver<(String, Ban)>>,
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
        Db {
            client:    Client::new(),
            nick:      env::get("IRC_NICK").to_lowercase(),
            owner_:    owner.to_lowercase(),
            owner,
            choices:   Vec::new(),
            reminders: MultiMap::new(),
            silences:  LocalMap::new(),
            tells:     MultiMap::new(),
            wiki:      Wikidot::new(),

            bans:      MultiMap::new(),
            bans_r:    None,
            titles:    HashMap::new(),
            titles_r:  None,

            pool
        }
    }

    pub fn conn(&self) -> Result<Conn, r2d2::Error> {
        self.pool.get()
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
                        match self.conn() {
                            Ok(conn) => diesel::update
                                (page::table.filter(page::id.eq(&k)).filter(page::title.ne(&title)))
                                .set(page::title.eq(&title))
                                .execute(&conn)
                                .log(trace!()),
                            err => err.log(trace!())
                        };
                        
                        self.titles.insert(k, v);
                    }
                }
            }
        }
        if let Some(bans_r) = &self.bans_r {
            loop {
                match bans_r.try_recv() {
                    Err(Empty)          => break,
                    Err(Disconnected)   => { self.bans_r = None; break },
                    Ok(((k, v), false)) => { util::multi_remove(&mut self.bans, &k, &v); },
                    Ok(((k, v), true))  => { self.bans.insert(k, v); }
                }
            }
        }
    }
    
    #[cfg(not(test))]
    fn retrieve<Frm, To, C, L, F>(&self, table: L, conn: &Conn, f: F) -> QueryResult<C>
    where C: FromIterator<To>, L: diesel::query_dsl::LoadQuery<PgConnection, Frm>, F: Fn(Frm) -> To {
        Ok(table.load::<Frm>(conn)?.into_iter().map::<To, F>(f).collect())
    }

    #[cfg(not(test))]
    pub fn reload(&mut self) -> IO<()> {
        let conn = self.conn()?;
        self.silences = silence::table.load(&conn)?.into_iter().collect();
        self.reminders = self.retrieve::<DbReminder,_,_,_,_>
            (reminder::table, &conn, |x| (x.user.to_owned(), Reminder::from(x)))?;
        self.tells = self.retrieve::<DbTell,_,_,_,_>
            (tell::table, &conn, |x| (x.target.to_owned(), Tell::from(x)))?;
        Ok(())
    }
    #[cfg(test)]
    pub fn reload(&mut self) -> IO<()> {
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

    pub fn get_ban(&self, ctx: &Context) -> Option<String> {
        let bans = self.bans.get_vec(&ctx.channel)?;
        let ban = bans.into_iter()
            .find(|x| x.active() && x.matches(&ctx.user, &ctx.host))?;
        Some(ban.reason.to_owned())
    }

    pub fn get_reminders(&mut self, ctx: &Context) -> Option<Vec<Reminder>> {
        let time = SystemTime::now();
        let mut reminders = self.reminders.get_vec_mut(&ctx.user)?;
        let expired = util::drain_filter(&mut reminders, |x| x.time < time);
        
        if !expired.is_empty() {
            match self.conn() {
                Ok(conn) => diesel::delete(
                    reminder::table
                        .filter(reminder::user.eq(&ctx.user))
                        .filter(reminder::time.lt(&time))
                    )
                    .execute(&conn)
                    .log(trace!()),
                err => err.log(trace!())
            }
        }
        
        Some(expired)
    }

    pub fn get_tells(&mut self, ctx: &Context) -> Option<Vec<Tell>> {
        let tells = self.tells.remove(&ctx.user)?;
        
        if !tells.is_empty() {
            match self.conn() {
                Ok(conn) => diesel::delete(tell::table.filter(tell::target.eq(&ctx.user)))
                    .execute(&conn)
                    .log(trace!()),
                err => err.log(trace!())
            }
        }
        
        Some(tells)
    }


    pub fn add_seen(&mut self, ctx: &Context, message: &str) -> IO<()> {
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
                    upsert(seen::latest),
                    upsert(seen::latest_time),
                    seen::total.eq(seen::total + 1)
                ))
            .execute(&self.conn()?)?;
        }
        Ok(())
    }

    pub fn get_seen(&self, channel: &str, nick: &str) -> IO<Seen> {
        Ok(seen::table
            .filter(seen::channel.eq(&channel.to_lowercase()))
            .filter(seen::user.eq(&nick.to_lowercase()))
        .first(&self.conn()?)?)
    }
}

pub fn upsert<T: Column + ExpressionMethods + Copy>(t: T) 
-> impl AsChangeset<Changeset=impl QueryFragment<Pg>, Target=T::Table> {
    t.eq(excluded(t))
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
