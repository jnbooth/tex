use std::time::SystemTime;

use super::*;

pub struct Tell;

impl<O: Output + 'static> Command<O> for Tell {
    fn cmds(&self) -> Vec<String> {
        abbrev("tell")
    }
    fn usage(&self) -> String { "<user> <message>".to_owned() }
    fn fits(&self, size: usize) -> bool { size > 1 }
    fn auth(&self) -> i32 { 0 }
    fn reload(&mut self, _: &mut Db) -> Outcome<()> { Ok(()) }

    fn run(&mut self, args: &[&str], irc: &O, ctx: &Context, db: &mut Db) -> Outcome<()> {
        let (nick, msg) = args.split_first().ok_or(InvalidArgs)?;
        add_tell(&msg.join(" "), nick, ctx, db)?;
        Ok(irc.action(ctx, &format!("writes down {}'s message for {}.", &ctx.nick, nick))?)
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
    #[cfg(not(test))] diesel
        ::insert_into(db::tell::table)
        .values(&tell)
        .execute(&db.conn)?;
    db.tells.insert(target, tell);
    Ok(())
}
