#[macro_use] extern crate diesel;

use chrono::Utc;
use diesel::prelude::*;
use hashbrown::HashSet;
use irc::client::prelude::*;
use std::iter::*;
use std::io;
use std::io::BufRead;
use std::thread;
use std::time::Duration;

#[macro_use] mod logging;
mod command;
mod context;
mod db;
mod error;
mod env;
mod output;
mod local;
mod handler;
mod wikidot; 

use self::context::Context;
use self::db::Db;
use self::command::Commands;
use self::logging::*;
use self::wikidot::{Wikidot, Diff, AuthorsDiff, PagesDiff, TitlesDiff};
pub use self::env::load;

#[macro_use] mod util;

#[cfg(test)]
pub const FUZZ: u16 = 100;

const CAPABILITIES: [Capability; 3] =
    [ Capability::ChgHost
    , Capability::ExtendedJoin
    , Capability::MultiPrefix
    ];

pub type IO<T> = Result<T, failure::Error>;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Api {
    user: String,
    key:  String
}

fn init(pool: db::Pool) -> IO<Db> {
    println!("Loading database...");
    let mut db = Db::new(pool.clone());
    println!("Loaded.");

    if env::opt("ATTRIBUTION_PAGE").is_some() {
        println!("Starting attribution scanner...");
        let (mut authors, authors_r) = AuthorsDiff::build(&pool)?;
        db.authors   = authors.cache().clone();
        db.authors_r = Some(authors_r);
        thread::spawn(move || {
            loop {
                if let Err(e) = authors.diff() {
                    log(WARN, &format!("Attribution error: {}", e));
                }
                thread::sleep(Duration::from_secs(3600));
            }
        });
        println!("Started.");
    }
    
    println!("Starting title scanner...");
    let (mut titles, titles_r) = TitlesDiff::build(&pool)?;
    db.titles   = titles.cache().clone().into_iter().collect();
    db.titles_r = Some(titles_r);
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(60));
            if let Err(e) = titles.diff() {
                log(WARN, &format!("Title error: {}", e))
            }
        }
    });
    println!("Started.");

    println!("Starting page scanner...");
    let (mut pages, pages_r) = PagesDiff::build(&pool)?;
    println!("Scanning...");
    db.download(pages.cache())?;
    db.loaded_r = Some(pages_r);
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(60));
            if let Err(e) = pages.diff() {
                log(WARN, &format!("Page error: {}", e));
            }
        }
    });
    println!("Started.");
    println!("Starting metadata scanner...");
    thread::spawn(move || {
        let cli = reqwest::Client::new();
        let mut wiki = Wikidot::new();
        loop {
            if let Err(e) = scan(&mut wiki, &cli, &pool) {
                log(WARN, &format!("Metadata error: {}", e));
            }
            thread::sleep(Duration::from_secs(300));
        }
    });
    println!("Started.");
    Ok(db)
}

fn scan(wiki: &mut Wikidot, cli: &reqwest::Client, pool: &db::Pool) -> IO<()> {
    let conn = pool.get()?;
    let start = Utc::now();
    let pages: Vec<String> = wiki.list(cli)?;
    wiki.walk(&pages.as_slice(), &cli, |title, page, tags: HashSet<String>| {
        diesel::update(db::page::table.filter(db::page::id.eq(&page.id)))
            .set(db::page::rating.eq(&page.rating))
            .execute(&conn)?;

        let oldtags: HashSet<String> = db::tag::table
            .filter(db::tag::page_id.eq(&page.id))
            .load(&conn)?
            .into_iter()
            .map(|x: db::Tag| x.name)
            .collect();

        for tag in oldtags.difference(&tags) {
            diesel::delete(
                db::tag::table
                .filter(db::tag::page_id.eq(&page.id))
                .filter(db::tag::name.eq(tag))
            ).execute(&conn)?;
        }

        for tag in tags.difference(&oldtags) {
            diesel::insert_into(db::tag::table)
                .values(db::Tag { name: tag.to_owned(), page_id: title.to_owned() })
                .on_conflict_do_nothing()
                .execute(&conn)?;
        }
        Ok(())
    })?;
    println!("Scanned in {}.", util::ago(start));
    Ok(())
}

pub fn run() -> IO<()> {
    let pool = db::establish_connection();
    let mut cmds = Commands::new(&pool);
    let mut db = init(pool)?;

    println!("Connecting...");
    let mut reactor = IrcReactor::new()?;
    let client = reactor.prepare_client_and_connect(&env::irc())?;
    client.send_cap_req(&CAPABILITIES)?;
    client.identify()?;
    println!("Connected.");

    reactor.
        register_client_with_handler(client, move |c, m| handler::handle(m, &mut cmds, c, &mut db));
    reactor.run()?;

    Ok(())
}

pub fn offline() -> IO<()> {
    let pool = db::establish_connection();
    let mut cmds = Commands::new(&pool);
    let mut db = init(pool)?;
    
    let client = output::Offline;
    
    println!("Awaiting input.");
    for line in io::stdin().lock().lines() {
        let message = format!(
            ":Jabyrwock!~jabyrwock@7B468DF6:FEE59C82:7ED85AB8:IP PRIVMSG #projectfreelancer :{}",
            line?
        ).parse()?;
        handler::handle(message, &mut cmds, &client, &mut db)?;
    }
    Ok(())
}
