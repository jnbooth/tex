use diesel::pg::PgConnection;
use irc::client::prelude::*;
use irc::error::IrcError;
use std::collections::HashMap;

use super::db;

mod wikipedia;

const COLOR_ECHO: u8 = 32;
const COLOR_DEBUG: u8 = 34;
const NO_RESULTS: &str = "I'm sorry, I couldn't find anything.";

pub fn respond(
    conn: &PgConnection, 
    props: &mut HashMap<String, String>, 
    client: &IrcClient, 
    source: &str, 
    target: &str, 
    message: &str
) -> Result<(), IrcError> {
    let (command, content) = match message.find(' ') {
        None => (message.to_lowercase(), "".to_string()),
        Some(i) => {
            let (command, content) = message.split_at(i);
            (command.to_lowercase(), content[1..].to_string())
        }
    };
    if command == "hug" {
        send_action(&client, target, &format!("hugs {}.", source))
    } else if command == "quit" {
        client.send_quit("Shutting down, bleep bloop.".to_owned())
    } else if command == "reload" {
        log(COLOR_DEBUG, "Reloading properties.");
        props.clear();
        for (k, v) in db::load_properties(conn) {
            props.insert(k, v);
        }
        send_privmsg(client, target, "Properties reloaded.")
    } else if "wikipedia".starts_with(&command) {
        match wikipedia::search(&content) {
            Ok(result) => send_reply(client, source, target, &result),
            Err(e) => {
                log(COLOR_DEBUG, &format!("Wikipedia error: {}", e));
                send_reply(client, source, target, NO_RESULTS)
            }
        }
    } else if "zyn".starts_with(&command) {
        send_reply(client, source, target, "Marp.")
    } else {
        Ok(())
    }
}

fn log(code: u8, s: &str) {
    println!("\x1b[{}m{}\x1b[0m", code, s)
}

fn send_action(client: &IrcClient, target: &str, msg: &str) -> Result<(), IrcError> {
    log(COLOR_ECHO, &format!("> /me {}", msg));
    client.send_action(target, msg)
}

fn send_privmsg(client: &IrcClient, target: &str, msg: &str) -> Result<(), IrcError> {
    log(COLOR_ECHO, &format!("> {}", msg));
    client.send_privmsg(target, msg)
}

fn send_reply(client: &IrcClient, source: &str, target: &str, msg: &str) -> Result<(), IrcError> {
    send_privmsg(client, target, &format!("{}: {}", source, msg))
}
