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

use diesel::pg::PgConnection;
use irc::client::prelude::*;
use irc::error::IrcError;
use std::collections::HashMap;
use std::iter::*;

mod db;
mod models;
mod response;
mod schema;

pub type IO<T> = Result<T, Box<std::error::Error>>;

pub fn run() {
    let conn = db::establish_connection();
    let mut props = db::load_properties(&conn);
    let config = to_config(&props);
    let mut reactor = IrcReactor::new().unwrap();
    let client = reactor.prepare_client_and_connect(&config).unwrap();
    client.identify().unwrap();

    reactor.register_client_with_handler(client, move |c, m| handler(&conn, &mut props, c, m));

    reactor.run().unwrap();
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

fn handler(
    conn: &PgConnection, 
    props: &mut HashMap<String, String>, 
    client: &IrcClient, message: Message
) -> Result<(), IrcError> {
    print!("{}", message);
    let m_prefix = message.prefix.to_owned();
    let m_target = message.response_target().to_owned();
    match (m_prefix, m_target, message.command.to_owned()) {
        (Some(prefix), Some(target), Command::PRIVMSG(_, msg)) => {
            let source = prefix.split("!").next().unwrap();
            for command in get_commands(&msg) {
                response::respond(&conn, props, &client, &source, &target, &command)?
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
