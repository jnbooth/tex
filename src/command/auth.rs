use super::*;

pub struct Auth;

impl<O: Output + 'static> Command<O> for Auth {
    fn cmds(&self) -> Vec<String> {
        own(&[&"auth"])
    }
    fn usage(&self) -> String { "<level> <user>".to_owned() }
    fn fits(&self, size: usize) -> bool { size == 2 }
    fn auth(&self) -> i32 { 3 }
    fn reload(&mut self, _: &mut Db) -> Outcome<()> { Ok(()) }

    fn run(&mut self, args: &[&str], irc: &O, ctx: &Context, db: &mut Db) -> Outcome<()> {
        let auth = parse_auth(args).ok_or(InvalidArgs)?;
        let nick = args.get(1).ok_or(InvalidArgs)?;
        if ctx.auth > auth && ctx.auth > db.auth(&nick) {
            add_user(&nick, auth, db)?;
            Ok(irc.action(ctx, &format!("promotes {} to rank {}.", nick, auth))?)
        } else {
            Ok(irc.reply(ctx, "Your authorization rank is not high enough to do that.")?)
        }
    }
}

fn parse_auth(args: &[&str]) -> Option<i32> {
    args.first()?.parse().ok()
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
