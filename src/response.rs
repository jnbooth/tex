use irc::client::prelude::*;
use irc::error::IrcError;

use super::db::Db;

mod wikipedia;

const COLOR_DEBUG: u8 = 34;
const COLOR_ECHO: u8 = 32;
const COLOR_WARN: u8 = 33;
const NO_RESULTS: &str = "I'm sorry, I couldn't find anything.";

pub fn respond(
    db: &mut Db, 
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
    if command == "auth" {
        match parse_auth(&content) {
            None => send_reply(client, target, source, &usage(&command)),
            Some((auth, nick)) => {
                if db.auth(auth + 1, source) && db.outranks(source, nick) {
                    log_db(db.add_user(auth, nick));
                    send_reply(client, target, source, &format!("Promoting {} to rank {}.", nick, auth))
                } else {
                    Ok(unauthorized(source, message))
                }
            }
        }
    } 

    else if command == "forget" {
        if db.auth(3, source) && db.outranks(source, &content) {
            match db.delete_user(&content) {
                Err(e) => 
                        Ok(log(COLOR_WARN, &format!("DB Error: {}", e))),
                Ok(true) => 
                        send_reply(client, target, source, &format!("Forgot {}.", content)),
                Ok(false) => 
                        send_reply(client, target, source, &format!("I don't know {}.", content)),
            }
        } else {
            Ok(unauthorized(source, message))
        }
    }

    else if command == "help" {
        send_reply(client, target, source, &usage(&content))
    }
    
    else if command == "hug" {
        send_action(client, target, &format!("hugs {}.", source))
    } 
    
    else if command == "quit" {
        if db.auth(3, source) {
            client.send_quit("Shutting down, bleep bloop.".to_owned())
        } else {
            Ok(unauthorized(source, message))
        }
    } 
    
    else if command == "reload" {
        if db.auth(4, source) {
            log(COLOR_DEBUG, "Reloading properties.");
            db.reload();
            send_privmsg(client, target, "Properties reloaded.")
        } else {
            Ok(unauthorized(source, message))
        }
    } 
    
    else if "wikipedia".starts_with(&command) {
        match wikipedia::search(&content) {
            Ok(result) => send_reply(client, target, source, &result),
            Err(e) => {
                log(COLOR_DEBUG, &format!("Wikipedia error: {}", e));
                send_reply(client, target, source, NO_RESULTS)
            }
        }
    } 
    
    else if "zyn".starts_with(&command) {
        send_reply(client, target, source, "Marp.")
    } 
    
    else {
        Ok(())
    }
}

fn usage(command: &str) -> String {
    let noargs = format!("Usage: [{}]", command);
    let args = |xs| format!("Usage: [{} {}]", command, xs);
    if command == "auth" {
        args("level user")
    } else if command == "forget" {
        noargs
    } else if command == "help" {
        args("command")
    } else if command == "hug" {
        noargs
    } else if command == "quit" {
        noargs
    } else if command == "reload" {
        noargs
    } else if "wikipedia".starts_with(&command) {
        args("article")
    } else if "zyn".starts_with(&command) {
        noargs
    } else {
        "I'm sorry, I don't know that command.".to_owned()
    }
}

fn log(code: u8, s: &str) {
    println!("\x1b[{}m{}\x1b[0m", code, s);
}

fn send_action(client: &IrcClient, target: &str, msg: &str) -> Result<(), IrcError> {
    log(COLOR_ECHO, &format!("> /me {}", msg));
    client.send_action(target, msg)
}

fn unauthorized(user: &str, command: &str) {
    log(COLOR_WARN, &format!("{} attempted to use an unauthorized command: {}!", user, command));
}

fn send_privmsg(client: &IrcClient, target: &str, msg: &str) -> Result<(), IrcError> {
    log(COLOR_ECHO, &format!("> {}", msg));
    client.send_privmsg(target, msg)
}

fn send_reply(client: &IrcClient, target: &str, source: &str, msg: &str) -> Result<(), IrcError> {
    send_privmsg(client, target, &format!("{}: {}", source, msg))
}

fn log_db(res: Result<(), diesel::result::Error>) {
    if let Err(e) = res {
        log(COLOR_WARN, &format!("DB error: {}", e));
    }
}

fn parse_auth(command: &str) -> Option<(i32, &str)> {
    let space = command.find(' ')?;
    let (auth_s, nick) = command.split_at(space);
    let auth: u16 = auth_s.parse().ok()?;
    Some((auth as i32, &nick[1..]))
}
