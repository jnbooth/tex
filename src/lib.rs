#[macro_use] extern crate diesel;

use irc::client::prelude::*;
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
            titles.diff().expect("Title thread error");
        }
    });
    println!("Started.");

    if let Some(wiki) = &db.wiki {
        println!("Starting page scanner...");
        let (mut pages, pages_r) = PagesDiff::build(wiki.clone())?;
        db.loaded_r = Some(pages_r);
        println!("Scanning...");
        db.download(&pages.dup())?;
        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_secs(60));
                pages.diff().expect("Title thread error");
            }
        });
        println!("Started.");
    }
    Ok(db)
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
