use super::*;
use crate::db::{Conn, memo, seen, tell};

pub struct Forget;

impl Command for Forget {
    fn cmds(&self) -> Vec<String> {
        own(&[&"forget"])
    }
    fn usage(&self) -> String { "<user>".to_owned() }
    fn fits(&self, size: usize) -> bool { size == 1 }
    fn auth(&self) -> u8 { 4 }

    fn run(&mut self, args: &[&str], _: &Context, db: &mut Db) -> Outcome {
        let nick = args[0];
        delete_user(&nick, &db.conn()?)?;
        Ok(vec![Action(format!("forgets {}.", nick))])
    }
}

fn delete_user(nick: &str, conn: &Conn) -> QueryResult<()> {
    let user = nick.to_lowercase();
    diesel::delete(memo::table.filter(memo::user.eq(&user))).execute(conn)?;
    diesel::delete(seen::table.filter(seen::user.eq(&user))).execute(conn)?;
    diesel::delete(tell::table.filter(tell::target.eq(&user))).execute(conn)?;
    diesel::delete(tell::table.filter(tell::sender.eq(&user))).execute(conn)?;
    Ok(())
}
