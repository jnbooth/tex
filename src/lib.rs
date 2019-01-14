#[macro_use] extern crate diesel;

use irc::client::prelude::*;
use std::io;
use std::io::BufRead;
use std::iter::*;
use std::thread;
use std::time::{Duration, SystemTime};

use self::db::Db;
use self::command::Commands;
use self::wikidot::titles::TitlesDiff;

mod command;
mod db;
mod error;
pub mod env;
mod output;
pub mod local;
mod logging;
mod handler;
mod wikidot; 

#[macro_use] pub mod util;

const CAPABILITIES: [Capability; 3] =
    [ Capability::ChgHost
    , Capability::ExtendedJoin
    , Capability::MultiPrefix
    ];

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

pub fn run() -> Result<(), failure::Error> {
    let mut cmds = Commands::new();
    let mut db = Db::new();
    let (mut titles, recv) = TitlesDiff::new()?;
    db.titles = titles.dup();
    db.titles_r = Some(recv);

    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(60));
            titles.diff().unwrap();
        }
    });

    let mut reactor = IrcReactor::new()?;
    let client = reactor.prepare_client_and_connect(&env::irc())?;
    client.send_cap_req(&CAPABILITIES).expect("Error negotiating capabilities");
    client.identify()?;

    reactor.
        register_client_with_handler(client, move |c, m| handler::handle(m, &mut cmds, c, &mut db));
    reactor.run()?;

    Ok(())
}

pub fn offline() -> Result<(), failure::Error> {
    let client = output::Offline;
    let mut cmds = Commands::new();
    let mut db = Db::new();
    let (mut titles, recv) = TitlesDiff::new()?;
    db.titles = titles.dup();
    db.titles_r = Some(recv);
    
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(60));
            titles.diff().unwrap();
        }
    });
    
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
