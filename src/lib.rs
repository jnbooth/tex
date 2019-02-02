#[macro_use] extern crate diesel;

use irc::client::prelude::*;
use std::io;
use std::io::BufRead;

#[macro_use] mod logging;
mod background;
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
use self::db::{Db, Pool, establish_connection};
use self::command::Commands;
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

fn init(pool: Pool) -> IO<Db> {
    let mut db = Db::new(pool.clone());
    background::spawn(pool, &mut db)?;
    Ok(db)
}

pub fn run() -> IO<()> {
    let pool = establish_connection();
    let mut cmds = Commands::new(&pool);
    let mut db = init(pool)?;

    let mut reactor = IrcReactor::new()?;
    let client = reactor.prepare_client_and_connect(&env::irc())?;
    client.send_cap_req(&CAPABILITIES)?;
    client.identify()?;

    reactor.
        register_client_with_handler(client, move |c, m| handler::handle(m, &mut cmds, c, &mut db));
    reactor.run()?;

    Ok(())
}

pub fn offline() -> IO<()> {
    let pool = establish_connection();
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
