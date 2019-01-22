#[macro_use] extern crate diesel;

use chrono::Utc;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use hashbrown::HashSet;
use irc::client::prelude::*;
use std::iter::*;
use std::io;
use std::io::BufRead;
use std::thread;
use std::time::Duration;

mod command;
mod context;
mod db;
mod error;
mod env;
mod output;
mod local;
mod logging;
mod handler;
mod wikidot; 

use self::context::Context;
use self::db::Db;
use self::command::Commands;
use self::logging::*;
use self::wikidot::Wikidot;
use self::wikidot::pages::PagesDiff;
use self::wikidot::titles::TitlesDiff;
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

fn init() -> IO<Db> {
    println!("Loading database...");
    let mut db = Db::new();
    println!("Loaded.");

    println!("Starting title scanner...");
    let (mut titles, titles_r) = TitlesDiff::build()?;
    db.titles = titles.dup();
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

    if let Some(wiki) = db.wiki.clone() {
        println!("Starting page scanner...");
        let (mut pages, pages_r) = PagesDiff::build(wiki.clone())?;
        db.loaded_r = Some(pages_r);
        println!("Scanning...");
        let pagelist = pages.dup();
        db.download(&pagelist)?;
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
            let conn = db::establish_connection();
            let cli = reqwest::Client::new();
            loop {
                if let Err(e) = scan(&wiki, &cli, &conn) {
                    log(WARN, &format!("Metadata error: {}", e));
                }
                thread::sleep(Duration::from_secs(5 * 60));
            }
        });
        println!("Started.");
    }
    Ok(db)
}

fn scan(wiki: &Wikidot, cli: &reqwest::Client, conn: &PgConnection) -> IO<()> {
    let start = Utc::now();
    let pages: Vec<String> = wiki.list(cli)?;
    wiki.walk(&pages.as_slice(), &cli, |title, page, tags: HashSet<String>| {
        diesel::update(db::page::table.filter(db::page::fullname.eq(&page.fullname)))
            .set(db::page::rating.eq(&page.rating))
            .execute(conn)?;

        let oldtags: HashSet<String> = db::tag::table
            .filter(db::tag::page.eq(&page.fullname))
            .load(conn)?
            .into_iter()
            .map(|x: db::Tag| x.name)
            .collect();

        for tag in oldtags.difference(&tags) {
            diesel::delete(
                db::tag::table
                .filter(db::tag::page.eq(&page.fullname))
                .filter(db::tag::name.eq(tag))
            ).execute(conn)?;
        }

        for tag in tags.difference(&oldtags) {
            diesel::insert_into(db::tag::table)
                .values(db::Tag { name: tag.to_owned(), page: title.to_owned() })
                .on_conflict_do_nothing()
                .execute(conn)?;
        }
        Ok(())
    })?;
    println!("Scanned in {}.", util::ago(start));
    Ok(())
}

pub fn run() -> IO<()> {
    let mut cmds = Commands::new();
    let mut db = init()?;

    println!("Connecting...");
    let mut reactor = IrcReactor::new()?;
    let client = reactor.prepare_client_and_connect(&env::irc())?;
    client.send_cap_req(&CAPABILITIES).expect("Error negotiating capabilities");
    client.identify()?;
    println!("Connected.");

    reactor.
        register_client_with_handler(client, move |c, m| handler::handle(m, &mut cmds, c, &mut db));
    reactor.run()?;

    Ok(())
}

pub fn offline() -> IO<()> {
    let client = output::Offline;
    let mut cmds = Commands::new();
    let mut db = init()?;
    
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
