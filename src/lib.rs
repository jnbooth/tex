#[macro_use] extern crate diesel;
extern crate dotenv;
extern crate failure;
extern crate humantime;
#[macro_use] extern crate lazy_static;
extern crate percent_encoding;
extern crate regex;
extern crate reqwest;
extern crate select;
extern crate serde;
extern crate serde_json;
extern crate xmlrpc;

use dotenv::dotenv;
use irc::client::prelude::*;
use irc::error::IrcError;
use percent_encoding::utf8_percent_encode;
use std::iter::*;

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

type IO<T> = Result<T, failure::Error>;

pub struct Api {
    user: String,
    key: String
}

pub fn run() -> Result<(), IrcError> {
    dotenv().ok();
    let mut db = Db::new();
    let config = get_config();
    let mut reactor = IrcReactor::new()?;
    let client = reactor.prepare_client_and_connect(&config)?;
    client.identify()?;

    reactor.register_client_with_handler(client, move |c, m| handler(&mut db, c, m));
    reactor.run()?;

    Ok(())
}

fn from_env(var: &str) -> String {
    std::env::var(var).expect(&format!("{} must be set in ./.env", var))
}

fn from_env_opt(var: &str) -> Option<String> {
    let res = std::env::var(var).ok()?.trim().to_owned();
    if res.is_empty() { None } else { Some(res) }
}

fn from_env_api(prefix: &str, user: &str, key: &str) -> Option<Api> {
    Some(Api { 
        user: from_env_opt(&format!("{}_{}", prefix, user))?, 
        key:  from_env_opt(&format!("{}_{}", prefix, key))? 
    })
}

fn encode(s: &str) -> String {
    utf8_percent_encode(s, percent_encoding::DEFAULT_ENCODE_SET).to_string()
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
            if let Some(source) = prefix.split("!").next() {
                let commands = get_commands(&msg);
                if commands.is_empty() {
                    print!("{}", message);
                } else {
                    log_part(color::ASK, &message.to_string());
                    if let Some(reminders) = db.get_reminders(source) {
                        for x in reminders {
                            client.privmsg(source, &format!("Reminder: {}", x.message))?
                      
                        }
                    } 
                    for command in commands {
                        response::respond(db, client, source, target, command)?
                    }
                }
            db::log(db.add_seen(&target, &source, &msg))
            }
        },
        _ => print!("{}", message)
    };
    Ok(())
}

fn get_commands(message: &str) -> Vec<&str> {
    match (message.chars().next(), message.get(1..)) {
        (Some('!'), Some(xs)) => vec![xs],
        (Some('.'), Some(xs)) => vec![xs],
        _ => message
            .split('[')
            .skip(1)
            .filter_map(|x| x.find(']').and_then(|i| {
                let cmd = x.split_at(i).0.trim();
                if cmd.is_empty() { None } else { Some(cmd) }
            }))
            .collect()
    }
}
