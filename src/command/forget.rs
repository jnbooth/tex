use super::*;

pub struct Forget;

impl Command for Forget {
    fn cmds(&self) -> Vec<String> {
        own(&[&"forget"])
    }
    fn usage(&self) -> String { "<user>".to_owned() }
    fn fits(&self, size: usize) -> bool { size == 1 }
    fn auth(&self) -> i32 { 3 }

    fn run(&mut self, args: &[&str], ctx: &Context, db: &mut Db) -> Outcome {
        let nick = args[0];
        if ctx.auth > db.auth(&nick) {
            delete_user(&nick, db)?;
            Ok(vec![Action(format!("forgets {}.", nick))])
        } else {
            Ok(vec![Reply("Your authorization rank is not high enough to do that.".to_owned())])
        }
    }
}

fn delete_user(nick: &str, db: &mut Db) -> QueryResult<bool> {
    let user = nick.to_lowercase();
    #[cfg(not(test))] {
        diesel
            ::delete(db::user::table.filter(db::user::nick.eq(&user)))
            .execute(&db.conn)?;
        diesel
            ::delete(db::memo::table.filter(db::memo::user.eq(&user)))
            .execute(&db.conn)?;
        diesel
            ::delete(db::seen::table.filter(db::seen::user.eq(&user)))
            .execute(&db.conn)?;
        diesel
            ::delete(db::tell::table.filter(db::tell::target.eq(&user)))
            .execute(&db.conn)?;
        diesel
            ::delete(db::tell::table.filter(db::tell::sender.eq(&user)))
            .execute(&db.conn)?;
    }
    let removed = db.users.remove(&user);
    Ok(removed.is_some())
}
