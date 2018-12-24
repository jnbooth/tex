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

use dotenv::dotenv;
use irc::client::prelude::*;
use irc::error::IrcError;
use std::env;
use std::iter::*;

mod db;
mod models;
mod response;
mod schema;
mod wikidot;

use self::db::*;

pub type IO<T> = Result<T, Box<std::error::Error>>;

pub fn run() -> Result<(), IrcError> {
    dotenv().ok();
    let mut db = Db::new();
    let config = get_config();
    let mut reactor = IrcReactor::new()?;
    let client = reactor.prepare_client_and_connect(&config)?;
    client.identify()?;

    reactor.register_client_with_handler(client, move |c, m| handler(&mut db, c, m));

    reactor.run()
}

fn from_env(var: &str) -> String {
    env::var(var).expect(&format!("{} must be set in ./.env", var))
}

fn get_config() -> Config {
    Config {
        server:   Some(from_env("IRC_SERVER")),
        nickname: Some(from_env("IRC_NICK")),
        password: Some(from_env("IRC_PASSWORD")),
        channels: Some(from_env("AUTOJOIN").split(",").map(|x| format!("#{}", x)).collect()),
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
