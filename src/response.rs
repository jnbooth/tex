use irc::error::IrcError;
use std::borrow::ToOwned;
use std::time::SystemTime;
use rand::Rng;

pub use super::db::Db;
use super::color;
use super::color::log;

pub mod choice;
mod dictionary;
mod reminder;
pub mod responder;
mod wikipedia;

pub const NO_RESULTS: &str = "I'm sorry, I couldn't find anything.";

const ABBREVIATE: [&str; 6] = ["choose", "define", "remindme", "select", "wikipedia", "zyn"];

fn abbreviate(command: &str) -> &str {
    for abbr in ABBREVIATE.into_iter() {
        if abbr.starts_with(command) {
            return abbr
        }
    }
    command
}

pub fn respond<T: responder::Responder>(
    db: &mut Db, 
    client: &T, 
    source: &str, 
    target: &str, 
    message: &str
) -> Result<(), IrcError> {
    let (command_base, content) = match message.find(' ') {
        None    => (message.to_lowercase(), "".to_owned()),
        Some(i) => {
            let (command, content) = message.split_at(i);
            (command.to_lowercase(), content[1..].to_owned())
        }
    };

    let command = abbreviate(&command_base);

    if db.silenced(target, command) {
        return warn(&format!("{} attempted to use a silenced command: {}!", target, command))
    }

    let args: Vec<String> = content
        .split(' ')
        .map(ToOwned::to_owned)
        .filter(|x| !x.is_empty())
        .collect();
    let len = args.len();
    
    let reply  = |msg: &str| client.reply(target, source, msg);
    let wrong  = || reply(&usage(&command_base));
    let unauth = || warn(
        &format!("{} attempted to use an unauthorized command: {}!", source, command)
    );

    match command {

    "auth" => {
        if !db.auth(3, source) {
            unauth()
        } else if len != 2 {
            wrong()
        } else if let Ok(auth) = args[0].parse() {
            let nick = args[1].to_owned();
            if !db.outranks(source, &nick) {
                unauth()
            } else {
                log_db(db.add_user(auth, &nick));
                reply(&format!("Promoting {} to rank {}.", nick, auth))
            }
        } else {
            wrong()
        }
    },

    "choose" => {
        let opts: Vec<&str> = content.split(',').map(str::trim).collect();
        reply(
            opts[ rand::thread_rng().gen_range(0, opts.len()) ]
        )
    },

    "define" => {
        if len == 0 {
            wrong()
        } else if let Ok(result) = dictionary::search(&content) {
            reply(&result)
        } else {
            reply(NO_RESULTS)
        }
    },
    
    "disable" => {
        if !db.auth(2, source) {
            unauth()
        } else if len != 1 {
            wrong()
        } else {
            let disable = abbreviate(&content);
            log_db(db.set_enabled(target, disable, false));
            reply(&format!("[{}] disabled.", disable))
        }
    },

    "enable" => {
        if !db.auth(2, source) {
            unauth()
        } else if len != 1 {
            wrong()
        } else {
            let enable = abbreviate(&content);
            log_db(db.set_enabled(target, enable, true));
            reply(&format!("[{}] enabled.", enable))
        }
    },
    
    "forget" => {
        if !db.auth(3, source) || !db.outranks(source, &content) {
            unauth()
        } else if len != 1 {
            wrong()
        } else { 
            match db.delete_user(&content) {
                Err(e)    => warn(&format!("DB Error: {}", e)),
                Ok(true)  => reply(&format!("Forgot {}.", content)),
                Ok(false) => reply(&format!("I don't know {}.", content)),
            }
        }
    },

    "help" => {
        if len != 1 {
            wrong()
        } else {
            reply(&usage(&content))
        }
    },

    "hug" => {
        if len != 0 {
            wrong()
        } else {
            client.action(target, &format!("hugs {}.", source))
        }
    },

    "quit" => {
        if !db.auth(3, source) {
            unauth()
        } else if len != 0 {
            wrong()
        } else {
            client.quit("Shutting down, bleep bloop.")
        }
    }, 

    "reload" => {
        if !db.auth(4, source) {
            unauth()
        } else if len != 0 {
            wrong()
        } else {
            log(color::DEBUG, "Reloading properties.");
            db.reload();
            reply("Properties reloaded.")
        }
    },

    "remindme" => {
        if len < 2 {
            wrong()
        } else if let Some(offset) = reminder::parse_offset(&args[0]) {
            let when = SystemTime::now() + offset;
            log_db(db.add_reminder(source, when, &args[1..].join(" ")));
            reply("Reminder added.")
        } else {
            wrong()
        }
    },

    "select" => {
        match content.parse() {
            Err(_) => wrong(),
            Ok(0)  => wrong(),
            Ok(i)  => match db.choices.run_choice(i) {
                Ok(result) => reply(&result),
                Err(_)     => reply(NO_RESULTS)
            }
        }
    },

    "wikipedia" => {
        if len == 0 {
            wrong()
        } else if let Ok(result) = wikipedia::search(db, &content) {
            reply(&result)
        } else {
            reply(NO_RESULTS)
        }
    },
    
    "zyn" => {
        reply("Marp.")
    },

    _ => Ok(())
    }
}

fn usage(command: &str) -> String {
    let noargs = format!("Usage: \x02{}\x02.", command);
    let args = |xs| format!("Usage: \x02{}\x02 {}.", command, xs);
    if command == "auth" {
        args("level user")
    } else if "choose".starts_with(command) {
        args("choices, separated, by commas")
    } else if "define".starts_with(command) {
        args("word")
    } else if command == "disable" {
        args("command")
    } else if command == "enable" {
        args("command")
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
    } else if "select".starts_with(&command) {
        args("number")
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

fn warn(msg: &str) -> Result<(), IrcError> {
    Ok(log(color::WARN, msg))
}

fn log_db(res: Result<(), diesel::result::Error>) {
    if let Err(e) = res {
        log(color::WARN, &format!("DB error: {}", e));
    }
}
