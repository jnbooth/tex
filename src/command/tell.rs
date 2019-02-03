use std::time::SystemTime;

use super::*;
use crate::db::tell;

pub struct Tell;

impl Command for Tell {
    fn cmds(&self) -> Vec<String> {
        abbrev("tell")
    }
    fn usage(&self) -> String { "<user> <message>".to_owned() }
    fn fits(&self, size: usize) -> bool { size >= 2 }
    fn auth(&self) -> u8 { 0 }

    fn run(&mut self, args: &[&str], ctx: &Context, db: &mut Db) -> Outcome {
        let (nick, msg) = args.split_first().unwrap();
        let target = nick.to_lowercase();
        let tell = db::Tell {
            sender:  ctx.nick.to_owned(),
            target:  target.to_owned(),
            time:    SystemTime::now(),
            message: msg.join(" ")
        };
        diesel::insert_into(tell::table).values(&tell).execute(&db.conn()?)?;
        db.tells.insert(target, tell);
        Ok(vec![Action(format!("writes down {}'s message for {}.", &ctx.nick, nick))])
    }
}
