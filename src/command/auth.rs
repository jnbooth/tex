use super::*;

pub struct Auth;

impl Command for Auth {
    fn cmds(&self) -> Vec<String> {
        own(&[&"auth"])
    }
    fn usage(&self) -> String { "<level> <user>".to_owned() }
    fn fits(&self, size: usize) -> bool { size == 2 }
    fn auth(&self) -> i32 { 3 }

    fn run(&mut self, args: &[&str], ctx: &Context, db: &mut Db) -> Outcome {
        let auth = args[0].parse().map_err(|_| InvalidArgs)?;
        let nick = args[1];
        if ctx.auth > auth && ctx.auth > db.auth(&nick) {
            add_user(&nick, auth, db)?;
            Ok(vec!(Action(format!("promotes {} to rank {}.", nick, auth))))
        } else {
            Ok(vec!(Action("Your authorization rank is not high enough to do that.".to_owned())))
        }
    }
}

pub fn add_user(nick: &str, auth: i32, db: &mut Db) -> QueryResult<()> {
    let obj = db::User {
        nick: nick.to_lowercase(),
        auth,
        pronouns: db.users.get(&nick.to_lowercase()).and_then(|x| x.pronouns.to_owned())
    };
    #[cfg(not(test))] diesel
        ::insert_into(db::user::table)
        .values(&obj)
        .on_conflict(db::user::nick)
        .do_update()
        .set(db::user::auth.eq(auth))
        .execute(&db.conn)?;
    db.users.insert(nick.to_lowercase(), obj);
    Ok(())
}
