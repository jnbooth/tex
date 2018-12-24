#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate percent_encoding;
#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate reqwest;
extern crate serde;
extern crate serde_json;
extern crate simple_error;
extern crate xmlrpc;

use irc::client::prelude::*;
use irc::error::IrcError;
use std::collections::HashMap;
use std::iter::*;

mod db;
mod models;
mod response;
mod schema;
mod wikidot;

use self::db::*;

pub type IO<T> = Result<T, Box<std::error::Error>>;

pub fn run() -> Result<(), IrcError> {
    let mut db = Db::new();
    let config = to_config(&db.props);
    let mut reactor = IrcReactor::new()?;
    let client = reactor.prepare_client_and_connect(&config)?;
    client.identify()?;

    reactor.register_client_with_handler(client, move |c, m| handler(&mut db, c, m));

    reactor.run()
}

fn to_config(props: &HashMap<String, String>) -> Config {
    Config {
        server: props.get("server").map(ToOwned::to_owned),
        nickname: props.get("nick").map(ToOwned::to_owned),
        channels: props.get("autojoin").map(|x| x.split(",").map(ToOwned::to_owned).collect()),
        password: props.get("password").map(ToOwned::to_owned),
        ..Config::default()
    }
}

fn handler(db: &mut Db, client: &IrcClient, message: Message) -> Result<(), IrcError> {
    print!("{}", message);
    let m_prefix = message.prefix.to_owned();
    let m_target = message.response_target().to_owned();
    match (m_prefix, m_target, message.command.to_owned()) {
        (Some(prefix), Some(target), Command::PRIVMSG(_, msg)) => {
            if let Some(source) = prefix.split("!").next() {
                for command in get_commands(&msg) {
                    response::respond(db, &client, &source, &target, &command)?
                }
            }
        },
        _ => ()
    };
    Ok(())
}

fn get_commands(message: &str) -> Vec<&str> {
    message
        .split('[')
        .skip(1)
        .filter_map(|x| x.find(']').and_then(|i| {
            let cmd = x.split_at(i).0.trim();
            if cmd.is_empty() { None } else { Some(cmd) }
        }))
        .collect()
}
