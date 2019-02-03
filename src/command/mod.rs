use diesel::prelude::*;
use failure::err_msg;
use stash::Stash;
use hashbrown::HashMap;

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

mod author;
mod search;

use crate::{Context, db, env};
use crate::db::{Db, Pool};
use crate::error::*;
use crate::logging::*;
use crate::output::{Output, Response};
use crate::output::Response::*;
use crate::util::own;

trait Command {
    fn cmds(&self) -> Vec<String>;
    fn usage(&self) -> String;
    fn auth(&self) -> u8;
    fn fits(&self, size: usize) -> bool;
    fn run(&mut self, args: &[&str], ctx: &Context, db: &mut Db) -> Outcome;
    
    #[cfg(test)]
    fn test(&mut self, query: &str, ctx: &Context, db: &mut Db) -> Result<String, Error> {
        let args: Vec<&str> = query.split(' ').filter(|x| !x.is_empty()).collect();
        let res = self.run(&args, ctx, db)?;
        let lines: Vec<String> = res.into_iter().map(|x| x.text().to_owned()).collect();
        Ok(lines.join("\n"))
    }
    #[cfg(test)]
    fn test_def(&mut self, query: &str) -> Result<String, Error> {
        self.test(query, &Context::default(), &mut Db::default())
    }
}

#[derive(Default)]
pub struct Commands {
    stash:  Stash<Box<dyn Command + 'static>, usize>,
    keys:   HashMap<String, usize>,
    canons: HashMap<String, String>,
    usages: HashMap<String, String>
}
impl Commands {
    pub fn new(pool: &Pool) -> Self {
        let mut x = Self::default();
        x.store(author::Author::new());
        x.store(choose::Choose::new());
        x.store(define::Define::new());
        x.store(forget::Forget);
        x.store(hug::Hug);
        x.store(lastcreated::LastCreated);
        x.store(quit::Quit);
        x.store(reload::Reload);
        x.store(remindme::Remindme::new());
        x.store(roll::Roll::new());
        x.store(search::Search::new());
        x.store(seen::Seen);
        x.store(tell::Tell);
        x.store(wikipedia::Wikipedia::new());
        x.store(zyn::Zyn);

        for &i in &[false, true] {
            x.store(memo::Memo::new(i));
            if let Some(g) = google::Google::build(i) {
                x.store(g);
            }
        }
        match name::Name::build(pool) {
            Err(e)    => log(ERROR, &format!("Error creating name command: {}", e)),
            Ok(names) => x.store(names)
        }
        for &i in &[false, true] {
            x.store(disable::Disable::new(i, x.canons.clone()));
        }
        x.usages.insert("help".to_owned(), "<command>".to_owned());
        x.usages.insert("h".to_owned(), "<command>".to_owned());
        x.usages.insert("showmore".to_owned(), "<number>".to_owned());
        x.usages.insert("sm".to_owned(), "<number>".to_owned());
        x
    }

    fn store<T: Command + 'static>(&mut self, t: T) {
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

    pub fn run<T: Output>(&mut self, cmd: &str, args: &[&str], ctx: &Context, db: &mut Db, irc: &T) 
    -> Outcome {
        if db.silences.contains(&ctx.channel, self.canons.get(cmd).ok_or(Unknown)?) {
            Err(Unauthorized)
        } else {
            match (cmd, args) {
                ("help", [query]) => { Ok(vec![Reply(self.usage(query))]) }
                ("help", _)       => Err(InvalidArgs),
                _ => {
                    let &key = self.keys.get(cmd).ok_or(Unknown)?;
                    let x = self.stash.get_mut(key).ok_or(Unknown)?;
                    
                    if x.auth() > db.auth(ctx, irc) {
                        Err(Unauthorized)
                    } else if !x.fits(args.len()) {
                        Err(InvalidArgs)
                    } else {
                        x.run(args, ctx, db)
                    }
                }
            }
        }
    }
}


fn abbrev(s: &str) -> Vec<String> {
    (0..s.len()).rev().map(|i| s[..=i].to_owned()).collect()
}
