use irc::client::prelude::*;
use irc::error::IrcError;
use regex::*;
use std::borrow::ToOwned;
use std::time::*;
use rand::*;

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
        None    => (message.to_lowercase(), "".to_string()),
        Some(i) => {
            let (command, content) = message.split_at(i);
            (command.to_lowercase(), content[1..].to_string())
        }
    };
    
    let wrong  = || send_reply(client, target, source, &usage(&command));
    let unauth = || Ok(unauthorized(source, message));

    let args: Vec<String> = content
        .split(' ')
        .map(ToOwned::to_owned)
        .filter(|x| !x.is_empty())
        .collect();
    let len = args.len();
    if command == "auth" {
        if len != 2 {
            wrong()
        } else { 
            match args[0].parse() {
                Err(_) => wrong(),
                Ok(auth) => {
                    let nick = args[1].to_owned();
                    if !db.auth(auth + 1, source) || db.outranks(source, &nick) {
                        unauth()
                    } else {
                        log_db(db.add_user(auth, &nick));
                        send_reply(client, target, source, &format!("Promoting {} to rank {}.", nick, auth))
                    }
                }
            }
        }
    }

    else if "choose".starts_with(&command) {
        let opts: Vec<&str> = content.split(',').map(str::trim).collect();
        let choice = opts[rand::thread_rng().gen_range(0, opts.len())];
        send_reply(client, target, source, choice)
    }

    else if command == "forget" {
        if !db.auth(3, source) || !db.outranks(source, &content) {
            unauth()
        } else if len != 1 {
            wrong()
        } else { 
            match db.delete_user(&content) {
                Err(e) => 
                        Ok(log(COLOR_WARN, &format!("DB Error: {}", e))),
                Ok(true) => 
                        send_reply(client, target, source, &format!("Forgot {}.", content)),
                Ok(false) => 
                        send_reply(client, target, source, &format!("I don't know {}.", content)),
            }
        }
    }

    else if command == "help" {
        if len != 1 {
            wrong()
        } else {
            send_reply(client, target, source, &usage(&content))
        }
    }
    
    else if command == "hug" {
        if len != 0 {
            wrong()
        } else {
            send_action(client, target, &format!("hugs {}.", source))
        }
    } 
    
    else if command == "quit" {
        if !db.auth(3, source) {
            unauth()
        } else if len != 0 {
            wrong()
        } else {
            client.send_quit("Shutting down, bleep bloop.".to_owned())
        }
    } 
    
    else if command == "reload" {
        if !db.auth(4, source) {
            unauth()
        } else if len != 0 {
            wrong()
        } else {
            log(COLOR_DEBUG, "Reloading properties.");
            db.reload();
            send_privmsg(client, target, "Properties reloaded.")
        }
    } 

    else if "remindme".starts_with(&command) {
        if len < 2 {
            wrong()
        } else {
            match parse_offset(&args[0]) {
                None         => wrong(),
                Some(offset) => {
                    let when = SystemTime::now() + offset;
                    log_db(db.add_reminder(source, when, &args[1..].join(" ")));
                    send_reply(client, target, source, "Reminder added.")
                }
            }
        }
    }
    
    else if "wikipedia".starts_with(&command) {
        if len == 0 {
            wrong()
        } else {
            match wikipedia::search(&content) {
                Ok(result) => send_reply(client, target, source, &result),
                Err(e) => {
                    log(COLOR_DEBUG, &format!("Wikipedia error: {}", e));
                    send_reply(client, target, source, NO_RESULTS)
                }
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
    let noargs = format!("Usage: {}.", command);
    let args = |xs| format!("Usage: {} {}.", command, xs);
    if command == "auth" {
        args("level user")
    } else if "choose".starts_with(command) {
        args("choices, separated, by commas")
    } else if command == "forget" {
        args("user")
    } else if command == "help" {
        args("command")
    } else if command == "hug" {
        noargs
    } else if command == "quit" {
        noargs
    } else if command == "reload" {
        noargs
    } else if "remindme".starts_with(&command) {
        format!("Usage: {} [<days>d][<hours>h][<minutes>m] message. Example: [{} 4h30m Fix my voice filter.]", command, command)
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

pub fn send_privmsg(client: &IrcClient, target: &str, msg: &str) -> Result<(), IrcError> {
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

fn yield_offset(d: u32, h: u32, m: u32) -> Option<Duration> {
    println!("{}d{}h{}m", d, h, m);
    Some(Duration::from_secs(60 * (m + 60 * (h + 24 * d)) as u64))
}

fn next<'r, 't>(groups: &mut Matches<'r, 't>) -> Option<u32> {
    groups.next()?.as_str().parse().ok()
}

fn parse_offset(s: &str) -> Option<Duration> {
    lazy_static! {
        static ref RE: Regex = Regex::new("\\d+").unwrap();
    }
    let format: &str = &RE.replace_all(s, "*").into_owned();
    let mut groups = RE.find_iter(s);
    match format {
        "*d*h*m" => yield_offset(next(&mut groups)?, next(&mut groups)?, next(&mut groups)?),
        "*d*h"   => yield_offset(next(&mut groups)?, next(&mut groups)?, 0),
        "*d*m"   => yield_offset(next(&mut groups)?, 0,                  next(&mut groups)?),
        "*d"     => yield_offset(next(&mut groups)?, 0,                  0),
        "*h*m"   => yield_offset(0,                  next(&mut groups)?, next(&mut groups)?),
        "*h"     => yield_offset(0,                  next(&mut groups)?, 0),
        "*m"     => yield_offset(0,                  0,                  next(&mut groups)?),
        _        => None
    }
}
