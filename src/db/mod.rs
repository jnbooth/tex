use diesel::prelude::*;
use diesel::query_dsl::{LoadQuery, RunQueryDsl};
use diesel::pg::PgConnection;
use diesel::pg::upsert::excluded;
use diesel::r2d2::ConnectionManager;
use hashbrown::{HashSet, HashMap};
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

use crate::wikidot::diff::DiffReceiver;
use crate::logging::Logged;
use crate::local::LocalMap;
use crate::{Context, IO, env, util};
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

    pub authors:   HashSet<String>,
    pub authors_r: Option<DiffReceiver<String>>,
    pub loaded:    HashSet<String>,
    pub loaded_r:  Option<DiffReceiver<String>>,
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

            authors:   HashSet::new(),
            loaded:    HashSet::new(),
            titles:    HashMap::new(),
            authors_r: None,
            loaded_r:  None,
            titles_r:  None,

            pool
        };
        db.reload().expect("Error loading database");
        db
    }

    pub fn conn(&self) -> Conn {
        self.pool.clone().get().expect("Failed to get connection from database pool")
    }

    pub fn listen(&mut self) {
        if let Some(authors_r) = &self.authors_r {
            loop {
                match authors_r.try_recv() {
                    Err(Empty)        => break,
                    Err(Disconnected) => { self.authors_r = None; break },
                    Ok((k, true))     => { self.authors.insert(k); },
                    Ok((k, false))    => { self.authors.remove(&k); }
                }
            }
        }
        if let Some(loaded_r) = &self.loaded_r {
            loop {
                match loaded_r.try_recv() {
                    Err(Empty)        => break,
                    Err(Disconnected) => { self.titles_r = None; break },
                    Ok((k, true))     => { self.loaded.insert(k); },
                    Ok((k, false))    => { self.loaded.remove(&k); }
                }
            }
        }
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
    
    fn retrieve<Frm, To, C, L, F>(&self, table: L, f: F) -> QueryResult<C>
    where C: FromIterator<To>, L: LoadQuery<PgConnection, Frm>, F: Fn(Frm) -> To {
        Ok(table.load::<Frm>(&self.conn())?.into_iter().map::<To, F>(f).collect())
    }

    pub fn reload(&mut self) -> QueryResult<()> {
        let conn = self.conn();
        #[cfg(not(test))] { self.bans = Bans::build(); }
        self.loaded = page::table.select(page::id).load(&conn)?.into_iter().collect();
        self.silences = silence::table.load(&conn)?.into_iter().collect();
        self.reminders = self.retrieve::<DbReminder,_,_,_,_>
            (reminder::table, |x| (x.user.to_owned(), Reminder::from(x)))?;
        self.tells = self.retrieve::<DbTell,_,_,_,_>
            (tell::table, |x| (x.target.to_owned(), Tell::from(x)))?;
        Ok(())
    }

    pub fn auth(&self, nick: &str) -> i32 {
        let user = nick.to_lowercase();
        if user == self.nick {
            5
        } else if user == self.owner_ {
            4
        } else {
            0
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
                    seen::latest.eq(message),
                    seen::latest_time.eq(SystemTime::now()),
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

    fn download_diff(&mut self, added: Vec<String>, deleted: Vec<String>) -> IO<()> {
        let conn = self.conn();
        for x in deleted {
            diesel::delete(page::table.filter(page::id.eq(&x))).execute(&conn)?;
            self.loaded.remove(&x);
        }
        let titles = self.titles.clone();
        let mut pages = Vec::new();
        let mut tags = Vec::new();
        self.wiki.walk(&added, &self.client, |title, mut page, pagetags: Vec<String>| {
            if let Some(title) = titles.get(&page.id) {
                page.title.push_str(": ");
                page.title.push_str(title);
            }
            pages.push(page);
            for tag in pagetags {
                tags.push(Tag { name: tag, page_id: title.to_owned() });
            }
            Ok(())
        })?;
        for chunk in pages.chunks(10_000) {
            diesel::insert_into(page::table)
                .values(chunk)
                .on_conflict(page::id)
                .do_update()
                .set(page::rating.eq(excluded(page::rating)))
                .execute(&conn)?;
        }
        for chunk in tags.chunks(20_000) {
            diesel::insert_into(tag::table)
                .values(chunk)
                .on_conflict_do_nothing()
                .execute(&conn)?;
        }
        for x in added {
            self.loaded.insert(x);
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
}

pub fn establish_connection() -> Pool {
    let manager = ConnectionManager::new(env::get("DATABASE_URL"));
    r2d2::Pool::builder()
        .max_size(env::get("DATABASE_POOL").parse().expect("Invalid DATABASE_POOL number"))
        .build(manager)
        .expect("Error connecting to database")
}
