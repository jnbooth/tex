use super::*;

pub struct Memo {
    shortcut: bool
}

impl Command for Memo {
    fn cmds(&self) -> Vec<String> {
        if self.shortcut { own(&["rem"]) } else { own(&["memo"]) }
    }
    fn usage(&self) -> String { 
        if self.shortcut {
            "<user> <message>".to_owned()
        } else {
            "<user> | memo (add|append|del) <user> <message>".to_owned() 
        }
    }   
    fn fits(&self, i: usize) -> bool { 
        if self.shortcut { i >= 2 } else { i != 2 }
    }
    fn auth(&self) -> i32 { 0 }

    fn run(&mut self, args: &[&str], ctx: &Context, db: &mut Db) -> Outcome {
        if self.shortcut {
            let (nick, msg) = args.split_first().unwrap();
            let message = self.append(&msg.join(" "), &nick.to_lowercase(), ctx, db)?;
            Ok(vec![
                Action(attribute(nick, ctx)),
                Reply(format!("New memo: \x1d{}\x1d", message))
            ])
        } else {
            let destructured: (&[&str], &[&str]) = 
                if args.len() < 2 { (args, &[]) } else { args.split_at(2) };
            match destructured {
                ([], _)     => 
                    Ok(vec![Reply(format!("\x1d{}\x1d", self.get(&ctx.user, ctx, db)?))]),
                ([nick], _) => 
                    Ok(vec![Reply(format!("\x1d{}\x1d", self.get(&nick.to_lowercase(), ctx, db)?))]),
                (["add", nick], msg) => match self.get(&nick.to_lowercase(), ctx, db) {
                    Ok(s) => Ok(vec![Reply(format!(
                        "{} already has a memo. To delete it, use .memo del {} \x1d{}\x1d",
                        nick, nick, s
                    ))]),
                    Err(NoResults) => {
                        self.insert(&msg.join(" "), &nick.to_lowercase(), ctx, db)?;
                        Ok(vec![Action(attribute(nick, ctx))])
                    },
                    Err(e) => Err(e)
                },
                (["append", nick], msg) => {
                    let message = self.append(&msg.join(" "), &nick.to_lowercase(), ctx, db)?;
                    Ok(vec![
                        Action(attribute(nick, ctx)),
                        Reply(format!("New memo: \x1d{}\x1d", message))
                    ])
                },
                (["del", nick], msg) => match self.get(&nick.to_lowercase(), ctx, db) {
                    Err(NoResults) => Ok(vec![Reply(format!("{} doesn't have a memo.", nick))]),
                    Err(e)         => Err(e),
                    Ok(ref s) if msg != s.split(' ').collect::<Vec<&str>>().as_slice() => 
                        Ok(vec![Reply(
                            format!("To delete that memo, use .memo del {} \x1d{}\x1d", nick, s)
                        )]),
                    _ => {
                            self.remove(&nick.to_lowercase(), ctx, db)?;
                            Ok(vec![Action(format!("erases {}'s memo.", nick))])
                        }
                },
                _ => Err(InvalidArgs)
            }
        }
    }
}

impl Memo {
    pub fn new(shortcut: bool) -> Self {
        Self { shortcut }
    }

    pub fn append(&mut self, s: &str, user: &str, ctx: &Context, db: &Db) -> Result<String, Error> {
        let message = match self.remove(user, ctx, db) {
            Ok(message)    => Ok(format!("{} {}", message, s)),
            Err(NoResults) => Ok(s.to_owned()),
            e              => e
        }?;
        self.insert(&message, user, ctx, db)?;
        Ok(message)
    }

    pub fn get(&self, user: &str, ctx: &Context, db: &Db) -> Result<String, Error> { 
        Ok(
            db::memo::table
                .filter(db::memo::channel.eq(&ctx.channel))
                .filter(db::memo::user.eq(user))
            .first::<db::Memo>(&db.conn())?
            .message
        )
    }

    pub fn remove(&mut self, user: &str, ctx: &Context, db: &Db) -> Result<String, Error> {
        Ok(
            diesel::delete(db::memo::table
                .filter(db::memo::channel.eq(&ctx.channel))
                .filter(db::memo::user.eq(user)))
            .returning(db::memo::message)
            .get_result(&db.conn())?
        )
    }

    pub fn insert(&mut self, message: &str, user: &str, ctx: &Context, db: &Db) -> Result<(), Error> {
        let memo = db::Memo { 
            channel: ctx.channel.to_owned(),
            user:    user.to_owned(),
            message: message.to_owned()
        };
        diesel::insert_into(db::memo::table)
            .values(&memo)
            .on_conflict((db::memo::channel, db::memo::user))
            .do_update()
            .set(db::memo::message.eq(message))
            .execute(&db.conn())?;
        Ok(())
    }
}

fn attribute(nick: &str, ctx: &Context) -> String {
    if nick.to_lowercase() == ctx.user {
        format!("writes down {}'s memo.", nick)
    } else {
        format!("writes down {}'s memo from {}.", nick, ctx.nick)
    }
}
