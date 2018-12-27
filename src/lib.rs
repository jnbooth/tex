#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate percent_encoding;
#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate reqwest;
extern crate select;
extern crate serde;
extern crate serde_json;
extern crate simple_error;
extern crate xmlrpc;

use dotenv::dotenv;
use irc::client::prelude::*;
use irc::error::IrcError;
use std::iter::*;
use simple_error::SimpleError;

mod color;
mod db;
mod models;
mod responder;
mod response;
mod schema;
mod wikidot;

use self::color::log_part;
use self::db::Db;
use self::responder::Responder;

pub type IO<T> = Result<T, Box<std::error::Error>>;

pub fn ErrIO<T>(e: &str) -> IO<T> {
    Err(Box::new(SimpleError::new(e)))
}

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
    std::env::var(var).expect(&format!("{} must be set in ./.env", var))
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

fn handler<T: Responder>(db: &mut Db, client: &T, message: Message) -> Result<(), IrcError> {
    let m_prefix = message.prefix.to_owned();
    let m_target = message.response_target().to_owned();
    match (m_prefix, m_target, message.command.to_owned()) {
        (Some(prefix), Some(target), Command::PRIVMSG(_, msg)) => {
            let commands = get_commands(&msg);
            if commands.is_empty() {
                print!("{}", message);
            } else {
                log_part(color::ASK, &message.to_string());
                if let Some(source) = prefix.split("!").next() {
                    if let Some(reminders) = db.get_reminders(source) {
                        for x in reminders {
                            client.privmsg(source, &format!("Reminder: {}", x.message))?
                        }
                    }
                    for command in commands {
                        response::respond(db, client, source, target, command)?
                    }
                }
            }
        },
        _ => print!("{}", message)
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
