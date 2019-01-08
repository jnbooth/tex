#[macro_use] extern crate diesel;

use irc::client::prelude::*;
use irc::error::IrcError;
use irc::proto::Command::*;
use std::iter::*;

mod color;
mod db;
mod env;
mod responder;
mod response;
#[macro_use] mod util;

use crate::color::log_part;
use crate::db::Db;
use crate::responder::Responder;

type IO<T> = Result<T, failure::Error>;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Api {
    user: String,
    key:  String
}

pub fn run() -> Result<(), IrcError> {
    let mut db = Db::new();
    let mut reactor = IrcReactor::new()?;
    let client = reactor.prepare_client_and_connect(&env::irc())?;
    client.identify()?;

    reactor.register_client_with_handler(client, move |c, m| handler(&mut db, c, m));
    reactor.run()?;

    Ok(())
}

fn parse_msg(message: Message) -> Option<(String, String, String)> {
    let target = message.response_target()?.to_owned();
    let prefix = message.prefix?.to_owned();
    Some((
        target, 
        prefix.split('!').next()?.to_owned(),
        prefix.split('@').last()?.to_owned()
    ))
}
fn handler<T: Responder>(db: &mut Db, client: &T, message: Message) -> Result<(), IrcError> {
    let text = message.to_string();
    match parse_msg(message.clone()) {
        None => print!("{}", text),
        Some((target, source, host)) => {
            match message.command {
                JOIN(_, _, _) => {
                    match &db.bans {
                        None       => print!("{}", text),
                        Some(bans) => match bans.get_ban(&target, &source, &host) {
                            None         => print!("{}", text),
                            Some(reason) => {
                                log_part(color::WARN, &text);
                                client.ban(&target, &source, &reason)?;
                            }
                        }
                    }
                },
                PRIVMSG(_, msg) => {
                    for reminder in db.get_reminders(&source).into_iter().flatten() {
                        client.privmsg(&source, &format!("Reminder: {}", reminder.message))?;
                    }
                    for tell in db.get_tells(&source).into_iter().flatten() {
                        client.privmsg(&source, &format!(
                            "From {} at {}: {}", tell.sender, util::show_time(tell.time), tell.message
                        ))?;
                    }
                    let commands = get_commands(&msg);
                    if commands.is_empty() {
                        print!("{}", text);
                    } else {
                        log_part(color::ASK, &text);
                        for command in commands {
                            response::respond(db, client, &source, &target, command)?
                        }
                    }
                    db::log(db.add_seen(&target, &source, &msg));
                },
                _ => print!("{}", text)
            }
        }
    }
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
                let cmd = x[..i].trim();
                if cmd.is_empty() { None } else { Some(cmd) }
            }))
            .collect()
    }
}
