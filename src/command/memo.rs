#[cfg(test)] 
use crate::local::LocalMap;

use super::*;

//#[cfg(not(test))] pub struct Rem;
//#[cfg(test)]      pub struct Rem(LocalMap<Memo>);

pub struct Memo {
    shortcut: bool,
    #[cfg(test)]
    db: LocalMap<db::Memo>
}

impl<O: Output + 'static> Command<O> for Memo {
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
        if self.shortcut { i > 1 } else { i != 2 }
    }
    fn auth(&self) -> i32 { 0 }
    fn reload(&mut self, _: &mut Db) -> Outcome<()> { Ok(()) }

    fn run(&mut self, args: &[&str], irc: &O, ctx: &Context, db: &mut Db) -> Outcome<()> {
        let say = |msg: String| Ok(irc.reply(ctx, &msg)?);
        if self.shortcut {
            match args.split_first() {
                None => Err(InvalidArgs)?,
                Some((nick, msg)) => {
                    let message = self.append(&msg.join(" "), &nick.to_lowercase(), ctx, db)?;
                    irc.action(&ctx, &attribute(nick, ctx))?;
                    say(format!("New memo: \x1d{}\x1d", message))
                },
            }
        } else {
            let destructured: (&[&str], &[&str]) = 
                if args.len() < 2 { (args, &[]) } else { args.split_at(2) };
            match destructured {
                ([], _)     => say(format!("\x1d{}\x1d", self.get(&ctx.user, ctx, db)?)),
                ([nick], _) => say(format!("\x1d{}\x1d", self.get(&nick.to_lowercase(), ctx, db)?)),
                (["add", nick], msg) => match self.get(&nick.to_lowercase(), ctx, db) {
                    Ok(s) => say(format!(
                        "{} already has a memo. To delete it, use .memo del {} \x1d{}\x1d",
                        nick, nick, s
                    )),
                    Err(NoResults) => {
                        self.insert(&msg.join(" "), &nick.to_lowercase(), ctx, db)?;
                        Ok(irc.action(&ctx, &attribute(nick, ctx))?)
                    },
                    Err(e) => Err(e)
                },
                (["append", nick], msg) => {
                    let message = self.append(&msg.join(" "), &nick.to_lowercase(), ctx, db)?;
                    irc.action(&ctx, &attribute(nick, ctx))?;
                    say(format!("New memo: \x1d{}\x1d", message))
                },
                (["del", nick], msg) => match self.get(&nick.to_lowercase(), ctx, db) {
                    Err(NoResults) => say(format!("{} doesn't have a memo.", nick)),
                    Err(e)         => Err(e),
                    Ok(ref s) if msg != s.split(' ').collect::<Vec<&str>>().as_slice() => 
                        say(format!("To delete that memo, use .memo del {} \x1d{}\x1d", nick, s)),
                    _ => {
                            self.remove(&nick.to_lowercase(), ctx, db)?;
                            Ok(irc.action(&ctx, &format!("erases {}'s memo.", nick))?)
                        }
                },
                _ => Err(InvalidArgs)
            }
        }
    }
}

impl Memo {
    pub fn new(shortcut: bool) -> Self {
        Memo {
            shortcut,
            #[cfg(test)]
            db: LocalMap::new()
        }
    }

    pub fn append(&mut self, s: &str, user: &str, ctx: &Context, db: &Db) -> Outcome<String> {
        let message = match self.remove(user, ctx, db) {
            Ok(message)    => Ok(format!("{} {}", message, s)),
            Err(NoResults) => Ok(s.to_owned()),
            e              => e
        }?;
        self.insert(&message, user, ctx, db)?;
        Ok(message)
    }

    #[cfg(not(test))]
    pub fn get(&self, user: &str, ctx: &Context, db: &Db) -> Outcome<String> {
        println!("{}", user);
        Ok(db::memo::table
            .filter(db::memo::channel.eq(&ctx.channel))
            .filter(db::memo::user.eq(user))
            .first::<db::DbMemo>(&db.conn)
            .map(|x| x.message)?
        )
    }

    #[cfg(not(test))]
    pub fn remove(&mut self, user: &str, ctx: &Context, db: &Db) -> Outcome<String> {
        Ok(diesel
            ::delete(db::memo::table
                .filter(db::memo::channel.eq(&ctx.channel))
                .filter(db::memo::user.eq(user))
            ).returning(db::memo::message)
            .get_result(&db.conn)?
        )
    }

    #[cfg(not(test))]
    pub fn insert(&mut self, message: &str, user: &str, ctx: &Context, db: &Db) -> Outcome<()> {
        let memo = db::Memo { 
            channel: ctx.channel.to_owned(),
            user:    user.to_owned(),
            message: message.to_owned()
        };
        diesel
            ::insert_into(db::memo::table)
            .values(&memo)
            .on_conflict((db::memo::channel, db::memo::user))
            .do_update()
            .set(db::memo::message.eq(message))
            .execute(&db.conn)?;
        Ok(())
    }
    
    #[cfg(test)]
    pub fn get(&self, user: &str, ctx: &Context, _: &Db) -> Outcome<String> {
        Ok(self.db.get(&ctx.channel, user).ok_or(NoResults)?.message.to_owned())
    }

    #[cfg(test)]
    pub fn remove(&mut self, user: &str, ctx: &Context, _: &Db) -> Outcome<String> {
        Ok(self.db.remove(&ctx.channel, user).ok_or(NoResults)?.message)
    }

    #[cfg(test)]
    pub fn insert(&mut self, message: &str, user: &str, ctx: &Context, _: &Db) -> Outcome<()> {
        let memo = db::Memo { 
            channel: ctx.channel.to_owned(),
            user:    user.to_owned(),
            message: message.to_owned()
        };
        self.db.insert(memo);
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
