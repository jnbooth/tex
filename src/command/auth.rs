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
        if ctx.auth > auth && (ctx.user == nick.to_lowercase() || ctx.auth > db.auth(&nick)) {
            add_user(&nick, auth, db)?;
            Ok(vec![Action(format!("promotes {} to rank {}.", nick, auth))])
        } else {
            Err(Unauthorized)
        }
    }
}

pub fn add_user(nick: &str, auth: i32, db: &mut Db) -> QueryResult<()> {
    let obj = db::User {
        nick: nick.to_lowercase(),
        auth,
        pronouns: db.users.get(&nick.to_lowercase()).and_then(|x| x.pronouns.to_owned())
    };
    db.execute(diesel
        ::insert_into(db::user::table)
        .values(&obj)
        .on_conflict(db::user::nick)
        .do_update()
        .set(db::user::auth.eq(auth))
    )?;
    db.users.insert(nick.to_lowercase(), obj);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn promotes_user() {
        let mut db = Db::default();
        Auth.run(&["1", "Foo"], &Context::admin(), &mut db).unwrap();
        assert_eq!(db.auth("Foo"), 1);
        Auth.run(&["2", "Foo"], &Context::admin(), &mut db).unwrap();
        assert_eq!(db.auth("Foo"), 2);
    }

    #[test]
    fn cannot_promote_to_own_auth() {
        assert!(Auth.run(&["5", "Foo"], &Context::admin(), &mut Db::default()).is_err())
    }

    #[test]
    fn cannot_demote_unless_outranking() {
        assert!(
            Auth.run(&["4", &Context::admin().nick], &Context::admin(), &mut Db::default()).is_err()
        )
    }

    #[test]
    fn can_demote_self() {
        let mut db = Db::default();
        let admin = Context::admin();
        let mut user = Context::default();
        Auth.run(&["4", &user.nick], &admin, &mut db).unwrap();
        user.auth = db.auth(&user.nick);
        Auth.run(&["1", &user.nick], &user, &mut db).unwrap();
        assert_eq!(db.auth(&user.nick), 1);
    }
}
