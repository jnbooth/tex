#[macro_use] extern crate diesel;

use irc::client::prelude::*;
use std::io;
use std::io::BufRead;
use std::iter::*;
use std::thread;
use std::time::{Duration, SystemTime};

mod command;
mod db;
mod error;
mod env;
mod output;
mod local;
mod logging;
mod handler;
mod wikidot; 

use self::db::Db;
use self::command::Commands;
use self::wikidot::pages::PagesDiff;
use self::wikidot::titles::TitlesDiff;
pub use self::env::load;

#[macro_use] mod util;

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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Context {
    pub channel: String,
    pub nick:    String,
    pub host:    String,
    pub user:    String,
    pub auth:    i32,
    pub time:    SystemTime
}
impl Context {
    pub fn new(db: &Db, message: Message) -> Option<Context> {
        let channel = message.response_target()?.to_lowercase();
        let prefix  = message.prefix?.to_owned();
        let nick    = prefix.split('!').next()?.to_owned();
        let host    = prefix.split('@').last()?.to_owned();
        let user    = nick.to_lowercase();
        let auth    = db.auth(&user);
        let time    = SystemTime::now();

        Some(Context { channel, nick, host, user, auth, time })
    }
    pub fn since(&self) -> String {
        match self.time.elapsed() {
            Err(_) => "now ".to_owned(),
            Ok(x)  => format!("{}.{:02}s ", x.as_secs(), x.subsec_millis() / 10)
        }
    }
    #[cfg(test)]
    pub fn mock(channel: &str, nick: &str) -> Context {
        Context { 
             channel: channel.to_lowercase(),
             nick:    nick.to_owned(),
             host:    String::new(),
             user:    nick.to_lowercase(),
             auth:    0,
             time:    SystemTime::now()
        }
    }
}

fn init() -> IO<Db> {
    let mut db = Db::new();

    let (mut titles, titles_r) = TitlesDiff::new()?;
    db.titles = titles.dup();
    db.titles_r = Some(titles_r);
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(60));
            titles.diff().expect("Title thread error");
        }
    });

    if let Some(wiki) = &db.wiki {
        let (mut pages, pages_r) = PagesDiff::new(wiki.clone())?;
        db.loaded_r = Some(pages_r);
        db.download(&pages.dup())?;
        thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(60));
            pages.diff().expect("Title thread error");
        }
    });
    }
    Ok(db)
}

pub fn run() -> IO<()> {
    let mut cmds = Commands::new();
    let mut db = init()?;

    let mut reactor = IrcReactor::new()?;
    let client = reactor.prepare_client_and_connect(&env::irc())?;
    client.send_cap_req(&CAPABILITIES).expect("Error negotiating capabilities");
    client.identify()?;

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

pub fn download() -> IO<()> {
    let mut db = Db::new();
    let (titles, _) = TitlesDiff::new()?;
    db.titles = titles.dup();
    db.download(&db.wiki.clone().expect("Error loading Wikidot").list(&db.client)?)
}
