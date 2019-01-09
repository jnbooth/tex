use irc::error::IrcError;
use std::time::SystemTime;
use rand::Rng;

use crate::{color, db, util};
use crate::db::Db;
use crate::color::log;
use crate::responder::Responder;
use crate::util::Gender;

pub mod choice;
mod dictionary;
mod google;
mod reminder;
mod roll;
mod seen;
pub mod wikidot;
mod wikipedia;

pub const NO_RESULTS: &str = "I'm sorry, I couldn't find anything.";

const ABBREVIATE: [&str; 9] =
        [ "choose"
        , "define"
        , "google"
        , "lastcreated"
        , "remindme"
        , "seen"
        , "tell"
        , "wikipedia"
        , "zyn"
        ];

fn abbreviate(command: &str) -> &str {
    match command {
        "lc" => "lastcreated",
        "sm" => "showmore",
        _    => {
            for abbr in ABBREVIATE.into_iter() {
                if abbr.starts_with(command) {
                    return abbr
                }
            }
            command
        }
    }
}

pub fn respond<T: Responder>(
    db: &mut Db, 
    client: &T, 
    source: &str, 
    target: &str, 
    message: &str
) -> Result<(), IrcError> {
    let (command_base, content) = util::split_on(" ", message).unwrap_or((message, ""));
    let owner = db.owner.to_owned();
    
    let reply  = |msg: &str| client.reply(target, source, msg);
    let warn   = |msg: &str| T::warn(&owner, msg);
    let wrong  = || match usage(&command_base) {
        None    => Ok(()),
        Some(s) => reply(&s)
    };
    let unauth = || {
        warn(&format!("{} used an unauthorized command: {}", source, command_base)); 
        Ok(()) 
    };
    let try_reply = |msg: Result<String, _>| match msg {
        Err(_) => reply(NO_RESULTS),
        Ok(s)  => reply(&s)
    };

    // Waiting for https://github.com/rust-lang/rust/issues/23121
    match (
        abbreviate(&command_base), 
        content.split(' ').filter(|x| !x.is_empty()).collect::<Vec<&str>>().as_slice()
    ) {
    (command, _) if db.silenced(target, command) => {
        warn(&format!("{} attempted to use a silenced command: {}!", target, command));
        Ok(())
    }

    ("auth", [auth_s, nick]) => {
        if !db.auth(3, source) {
            unauth()
        } else if let Ok(auth) = auth_s.parse() {
            if !db.outranks(source, &nick) {
                unauth()
            } else {
                db::log(db.add_user(auth, &nick));
                reply(&format!("Promoting {} to rank {}.", nick, auth))
            }
        } else {
            wrong()
        }
    },

    ("choose", []) => wrong(),
    ("choose", _)  => {
        let opts: Vec<&str> = content.split(',').map(str::trim).collect();
        reply(
            opts[ rand::thread_rng().gen_range(0, opts.len()) ]
        )
    },

    ("define", []) => wrong(),
    ("define", _)  => 
        try_reply(dictionary::search(&db.client, &content)),
    
    ("disable", [cmd]) => {
        if !db.auth(2, source) {
            unauth()
        } else {
            let disable = abbreviate(&cmd);
            db::log(db.set_enabled(target, disable, false));
            reply(&format!("[{}] disabled.", disable))
        }
    },

    ("enable", [cmd]) => {
        if !db.auth(2, source) {
            unauth()
        } else {
            let enable = abbreviate(&cmd);
            db::log(db.set_enabled(target, enable, true));
            reply(&format!("[{}] enabled.", enable))
        }
    },
    
    ("forget", [nick]) => {
        if !db.auth(3, source) || !db.outranks(source, &content) {
            unauth()
        } else { 
            reply(&match db.delete_user(&nick) {
                Err(e)    => warn(&format!("Error deleting user {}: {}", nick, e)),
                Ok(true)  => format!("Forgot {}.", nick),
                Ok(false) => format!("I don't know {}.", nick),
            })
        }
    },

    ("gis", []) => wrong(),
    ("gis", _)  => {
        match &db.api.google {
            None      => Ok(()),
            Some(api) => try_reply(google::search_image(&api, &db.client, &content))
        }
    },

    ("google", []) => wrong(),
    ("google", _)  => {
        match &db.api.google {
            None      => Ok(()),
            Some(api) => try_reply(google::search(&api, &db.client, &content))
        }
    },

    ("help", [cmd]) => match usage(&cmd) {
        None    => reply("I'm sorry, I don't know that command."),
        Some(s) => reply(&s)
    },

    ("hug", []) => 
        client.action(target, &format!("hugs {}.", source)),

    ("lastcreated", []) => {
        match &db.api.wikidot {
            None => Ok(()),
            Some(wikidot) => match wikidot.last_created(&db.client) {
                Err(e)    => reply(&warn(&format!(".lc error: {}", e))),
                Ok(pages) => { for page in pages { reply(&page)? } Ok(()) }
            }
        }
    }

    ("name", [])     => reply(&db.names.gen(Gender::Any)),
    ("name", ["-f"]) => reply(&db.names.gen(Gender::Female)),
    ("name", ["-m"]) => reply(&db.names.gen(Gender::Male)),

    ("quit", []) => {
        if !db.auth(3, source) {
            unauth()
        } else {
            client.quit("Shutting down, bleep bloop.")
        }
    }, 

    ("reload", []) => {
        if !db.auth(4, source) {
            unauth()
        } else {
            log(color::DEBUG, "Reloading properties.");
            match db.reload() {
                Err(e) => reply(&warn(&format!("Error reloading database: {}", e))),
                Ok(()) => reply("Properties reloaded.")
            }
        }
    },

    ("remindme", args) if args.len() >= 2 => {
        if let Some(offset) = reminder::parse_offset(&args[0]) {
            let when = SystemTime::now() + offset;
            db::log(db.add_reminder(source, when, &args[1..].join(" ")));
            reply("Reminder added.")
        } else {
            wrong()
        }
    },

    ("roll", []) => wrong(),
    ("roll", _)  => {
        if let Ok(result) = roll::throw(&content) {
            reply(&format!("{} (rolled {})", result, content))
        } else {
            reply("Invalid roll.")
        }
    },

    ("seen", args) => {
        match seen::search(db, target, &args) {
            Err(seen::Error::InvalidArgs) => wrong(),
            Err(seen::Error::NotFound)    => reply(NO_RESULTS),
            Ok(result)                    => reply(&result)
        }
    },

    ("showmore", [i_s]) => {
        match i_s.parse() {
            Err(_) => wrong(),
            Ok(0)  => wrong(),
            Ok(i)  => try_reply(db.choices.run_choice(i - 1))
        }
    },

    ("tell", args) if args.len() >= 2 => {
        db::log(db.add_tell(source, &args[0], &args[1..].join(" ")));
        client.action(target, &format!("writes down {}'s message and nods.", source))
    },

    ("wikipedia", []) => wrong(),
    ("wikipedia", _) => 
        try_reply(wikipedia::search(db, &content)),
    
    ("zyn", []) => {
        reply("Marp.")
    },

    _ => wrong()
    }
}

fn usage(command: &str) -> Option<String> {
    let noargs = Some(format!("Usage: \x02{}\x02.", command));
    let args = |xs| Some(format!("Usage: \x02{}\x02 {}.", command, xs));
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
    } else if command == "gis" {
        args("query")
    } else if "google".starts_with(command) {
        args("query")
    } else if command == "help" {
        args("command")
    } else if command == "hug" {
        noargs
    } else if command == "lc" || "lastcreated".starts_with(command) {
        noargs
    } else if command == "name" {
        args("[-f|-m]")
    } else if command == "quit" {
        noargs
    } else if command == "reload" {
        noargs
    } else if command == "roll" {
        Some("Usage examples: [roll d20 + 4 - 2d6!], [roll 3dF-2], [roll 2d6>3 - 1d4].".to_string())
    } else if "seen".starts_with(&command) {
        args("[#channel] [-f|-t] user")
    } else if command == "showmore" || command == "sm" {
        args("number")
    } else if "remindme".starts_with(&command) {
        Some(format!("Usage: {} [<days>d][<hours>h][<minutes>m] message. Example: [{} 4h30m Fix my voice filter.]", command, command))
    } else if "tell".starts_with(&command) {
        args("user message")
    } else if "wikipedia".starts_with(&command) {
        args("article")
    } else if "zyn".starts_with(&command) {
        noargs
    } else {
        None
    }
}
