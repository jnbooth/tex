use irc::client::prelude::*;
use irc::error::IrcError;
use irc::proto::Command::*;
use std::borrow::ToOwned;
use std::iter::*;

use crate::{Context, db, util};
use crate::command::Commands;
use crate::db::Db;
use crate::logging::*;
use crate::output::Output;
use crate::error::*;

pub const NO_RESULTS: &str = "I'm sorry, I couldn't find anything.";
const CHARACTER_LIMIT: usize = 429;

pub fn handle<O: Output>(message: Message, cmds: &mut Commands<O>, irc: &O, db: &mut Db) 
-> Result<(), IrcError> {
    db.listen();
    let text = message.to_string();
    match Context::build(db, message.to_owned()) {
        None      => print!("{}", text),
        Some(ctx) => {
            match message.command {
                JOIN(_, _, _) => {
                    match &db.bans {
                        None     => print!("{}", text),
                        Some(xs) => match xs.get_ban(&ctx) {
                            None         => print!("{}", text),
                            Some(reason) => {
                                log_part(WARN, &text);
                                irc.ban(&ctx, &reason)?;
                            }
                        }
                    }
                },
                PRIVMSG(_, msg) => {
                    for reminder in db.get_reminders(&ctx).into_iter().flatten() {
                        irc.pm(&ctx, &format!("Reminder: {}", reminder.message)
                        )?;
                    }
                    for tell in db.get_tells(&ctx).into_iter().flatten() {
                        irc.pm(&ctx, &format!(
                            "From {} at {}: {}", tell.sender, util::show_time(tell.time), tell.message
                        ))?;
                    }
                    let commands = get_commands(&msg);
                    if commands.is_empty() {
                        print!("{}", text);
                    } else {
                        log_part(ASK, &text);
                        for command in commands {
                            run(cmds, command, irc, &ctx, db)?
                        }
                    }
                    db::log(db.add_seen(&ctx, &msg));
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
        _                     => message
            .split('[')
            .skip(1)
            .filter_map(|x| x.find(']').and_then(|i| {
                let cmd = x[..i].trim();
                if cmd.is_empty() { None } else { Some(cmd) }
            }))
            .collect()
    }
}

fn suggest(suggestions: &[String]) -> String {
    if suggestions.is_empty() {
        NO_RESULTS.to_owned()
    } else {
        let mut s = "Did you mean:".to_owned();
        for (i, suggest) in suggestions.into_iter().enumerate() {
            if s.len() + suggest.len() + 7 > CHARACTER_LIMIT {
                return s.to_owned()
            }
            if i > 0 {
                s.push_str(",");
            }
            s.push_str(" (");
            s.push_str(&(i-1).to_string());
            s.push_str(") ");
            s.push_str(suggest);
        }
        s.to_owned()
    }
}

fn run<O: Output>(cmds: &mut Commands<O>, message: &str, irc: &O, ctx: &Context, db: &mut Db) 
-> Result<(), IrcError> {
    let (cmd, args): (String, Vec<&str>) = match util::split_on(" ", message) {
        None         => (message.to_lowercase(), Vec::new()),
        Some((x, y)) => (x.to_lowercase(), y.split(' ').filter(|x| !x.is_empty()).collect())
    };
    if cmd == "showmore" || cmd == "sm" {
        match args.as_slice() {
            [val] => match val.parse::<usize>() {
                Ok(i) if i > 0 => match db.choices.get(i - 1).map(ToOwned::to_owned) {
                    None    => irc.reply(ctx, "That isn't one of my options."),
                    Some(x) => run(cmds, &x, irc, ctx, db)
                },
                _ => irc.reply(ctx, &cmds.usage(&cmd))
            },
            _ => irc.reply(ctx, &cmds.usage(&cmd))
        }
    } else {
        match cmds.run(&cmd, &args, irc, ctx, db) {
            Err(IrcErr(e))   => Err(*e),
            Ok(())           => Ok(()),
            Err(Unknown)     => Ok(()),
            Err(InvalidArgs) => irc.reply(ctx, &cmds.usage(&cmd)),
            Err(NoResults)   => irc.reply(ctx, NO_RESULTS),
            Err(Ambiguous(size, xs)) => {
                db.choices.clear();
                for x in &xs {
                    db.choices.push(format!("{} {}", cmd, x));
                }
                match size {
                    0 => irc.reply(ctx, &suggest(&xs)),
                    _ => irc.reply(ctx, &format!("{} ({} total)", suggest(&xs), size))
                }
            },
            Err(Unauthorized) => {
                log(WARN, &format!("{} used an unauthorized command: {}", ctx.nick, cmd)); 
                Ok(()) 
            },
            Err(ParseErr(e)) => {
                log(DEBUG, &format!("Parse error for '{}': {}", message, e));
                irc.reply(ctx, NO_RESULTS)
            },
            Err(Throw(e)) => {
            log(DEBUG, &format!("Unhandled error for '{}': {}", message, e));
                match &db.owner {
                    None    => irc.reply(ctx, "Something went wrong."),
                    Some(s) => 
                        irc.reply(ctx, &format!("Something went wrong. Please let {} know.", s))
                }
            }
        }
    }
}
