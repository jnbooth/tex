use std::time::SystemTime;

use super::*;

pub struct Tell;

impl Command for Tell {
    fn cmds(&self) -> Vec<String> {
        abbrev("tell")
    }
    fn usage(&self) -> String { "<user> <message>".to_owned() }
    fn fits(&self, size: usize) -> bool { size >= 2 }
    fn auth(&self) -> i32 { 0 }

    fn run(&mut self, args: &[&str], ctx: &Context, db: &mut Db) -> Outcome {
        let (nick, msg) = args.split_first().unwrap();
        add_tell(&msg.join(" "), nick, ctx, db)?;
        Ok(vec![Action(format!("writes down {}'s message for {}.", &ctx.nick, nick))])
    }
}

fn add_tell(message: &str, target_nick: &str, ctx: &Context, db: &mut Db) -> QueryResult<()> {
    let target = target_nick.to_lowercase();
    let tell = db::Tell {
        sender:  ctx.nick.to_owned(),
        target:  target.to_owned(),
        time:    SystemTime::now(),
        message: message.to_owned()
    };
    db.execute(diesel::insert_into(db::tell::table).values(&tell))?;
    db.tells.insert(target, tell);
    Ok(())
}
