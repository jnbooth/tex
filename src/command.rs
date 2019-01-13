use diesel::prelude::*;
use failure::err_msg;
use stash::Stash;
use hashbrown::HashMap;

use crate::{Context, env, wikidot};
use crate::db;
use crate::db::Db;
use crate::error::*;
use crate::output::Output;
use crate::util::own;
#[cfg(not(test))] use crate::db::schema::*;

mod auth;
mod choose;
mod define;
mod disable;
mod forget;
mod google;
mod hug;
mod lastcreated;
mod memo;
mod name;
mod quit;
mod reload;
mod remindme;
mod roll;
mod seen;
mod tell;
mod wikipedia;
mod zyn;

trait Command<O: Output + 'static> {
    fn cmds(&self) -> Vec<String>;
    fn usage(&self) -> String;
    fn auth(&self) -> i32;
    fn fits(&self, size: usize) -> bool;
    fn reload(&mut self, db: &mut Db) -> Outcome<()>;
    fn run(&mut self, args: &[&str], irc: &O, ctx: &Context, db: &mut Db) -> Outcome<()>;
}


pub struct Commands<O: Output + 'static> {
    stash:  Stash<Box<dyn Command<O>>, usize>,
    keys:   HashMap<String, usize>,
    canons: HashMap<String, String>,
    usages: HashMap<String, String>
}
impl<O: Output + 'static> Commands<O> {
    pub fn new() -> Self {
        let mut x = Self::empty();
        x.store(auth::Auth);
        x.store(choose::Choose::new());
        x.store(define::Define::new());
        x.store(forget::Forget);
        x.store(hug::Hug);
        x.store(quit::Quit);
        x.store(reload::Reload);
        x.store(remindme::Remindme::new());
        x.store(roll::Roll::new());
        x.store(seen::Seen);
        x.store(tell::Tell);
        x.store(wikipedia::Wikipedia::new());
        x.store(zyn::Zyn);
        for &i in &[false, true] {
            x.store(memo::Memo::new(i));
            if let Some(g) = google::Google::new(i) {
                x.store(g);
            }
        }
        match name::Name::new() {
            Err(e)    => println!("Error creating name command: {}", e),
            Ok(names) => x.store(names)
        }
        if let Some(wiki) = wikidot::Wikidot::new() {
            x.store(lastcreated::LastCreated::new(wiki));
        }
        for &i in &[false, true] {
            x.store(disable::Disable::new(i, x.canons.to_owned()));
        }
        x.usages.insert("help".to_owned(), "<command>".to_owned());
        x.usages.insert("h".to_owned(), "<command>".to_owned());
        x.usages.insert("showmore".to_owned(), "<number>".to_owned());
        x.usages.insert("sm".to_owned(), "<number>".to_owned());
        x
    }

    pub fn empty() -> Self {
        Commands {
            stash:  Stash::default(),
            keys:   HashMap::new(),
            canons: HashMap::new(),
            usages: HashMap::new()
        }
    }

    fn store<T: Command<O> + 'static>(&mut self, t: T) {
        let cmds = t.cmds();
        let canon = cmds[0].to_owned();
        let usage = t.usage();
        let key = self.stash.put(Box::new(t));
        for cmd in cmds {
            self.usages.insert(cmd.to_owned(), usage.to_owned());
            self.canons.insert(cmd.to_owned(), canon.to_owned());
            self.keys.insert(cmd, key);
        }
    }

    pub fn usage(&self, cmd: &str) -> String {
        match self.usages.get(cmd) {
            None    => "I don't know that command.".to_owned(),
            Some(x) => format!("Usage: \x02{}\x02 {}", cmd, x)
        }
    }

    pub fn run(&mut self, cmd: &str, args: &[&str], irc: &O, ctx: &Context, db: &mut Db) 
    -> Outcome<()> {
        if db.silences.contains(&ctx.channel, self.canons.get(cmd).ok_or(Unknown)?) {
            Err(Unauthorized)
        } else {
            match (cmd, args) {
                ("help", [query]) => Ok(irc.reply(ctx, &self.usage(query))?),
                ("help", _)       => Err(InvalidArgs),
                _ => {
                    let key = self.keys.get(cmd).ok_or(Unknown)?;
                    let x = self.stash.get_mut(*key).ok_or(Unknown)?;
                    
                    if x.auth() > ctx.auth {
                        Err(Unauthorized)
                    } else if !x.fits(args.len()) {
                        Err(InvalidArgs)
                    } else {
                        x.run(args, irc, ctx, db)
                    }
                }
            }
        }
    }
}


fn abbrev(s: &str) -> Vec<String> {
    (0..s.len()).rev().map(|i| s[0..i+1].to_owned()).collect()
}
