use super::*;

pub struct Forget;

impl<O: Output + 'static> Command<O> for Forget {
    fn cmds(&self) -> Vec<String> {
        own(&[&"forget"])
    }
    fn usage(&self) -> String { "<user>".to_owned() }
    fn fits(&self, size: usize) -> bool { size == 1 }
    fn auth(&self) -> i32 { 3 }
    fn reload(&mut self, _: &mut Db) -> Outcome<()> { Ok(()) }

    fn run(&mut self, args: &[&str], irc: &O, ctx: &Context, db: &mut Db) -> Outcome<()> {
        let nick = args.get(0).ok_or(InvalidArgs)?;
        if ctx.auth > db.auth(&nick) {
            delete_user(&nick, db)?;
            Ok(irc.action(ctx, &format!("forgets {}.", nick))?)
        } else {
            Ok(irc.reply(ctx, "Your authorization rank is not high enough to do that.")?)
        }
    }
}

fn delete_user(nick: &str, db: &mut Db) -> QueryResult<bool> {
    let user = nick.to_lowercase();
    #[cfg(not(test))] {
        diesel
            ::delete(user::table.filter(user::nick.eq(&user)))
            .execute(&db.conn)?;
        diesel
            ::delete(db::schema::memo::table.filter(db::schema::memo::user.eq(&user)))
            .execute(&db.conn)?;
        diesel
            ::delete(db::schema::seen::table.filter(db::schema::seen::user.eq(&user)))
            .execute(&db.conn)?;
        diesel
            ::delete(db::schema::tell::table.filter(db::schema::tell::target.eq(&user)))
            .execute(&db.conn)?;
        diesel
            ::delete(db::schema::tell::table.filter(db::schema::tell::sender.eq(&user)))
            .execute(&db.conn)?;
    }
    let removed = db.users.remove(&user);
    Ok(removed.is_some())
}
