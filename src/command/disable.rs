use super::*;

pub struct Disable {
    enable: bool,
    canons: HashMap<String, String>
}

impl<O: Output + 'static> Command<O> for Disable {
    fn cmds(&self) -> Vec<String> {
        if self.enable { own(&["enable"]) } else { own(&["disable"]) }
    }
    fn usage(&self) -> String { "<command>".to_owned() }
    fn fits(&self, size: usize) -> bool { size == 1 }
    fn auth(&self) -> i32 { 2 }
    fn reload(&mut self, _: &mut Db) -> Outcome<()> { Ok(()) }

    fn run(&mut self, args: &[&str], irc: &O, ctx: &Context, db: &mut Db) -> Outcome<()> {
        let cmd = args.join(" ").to_lowercase();
        let canon = match cmd.chars().next() {
            Some('!') => self.canons.get(&cmd[1..]),
            Some('.') => self.canons.get(&cmd[1..]),
            _         => self.canons.get(&cmd)
        };
        match canon {
            None        => Ok(irc.reply(ctx, "I'm sorry, I don't know that command.")?),
            Some(canon) => {
                self.set_enabled(&canon, ctx, db)?;
                Ok(irc.action(ctx, 
                    &format!("{}s .{}.", if self.enable { "enable" } else { "disable" }, canon)
                )?)
            }
        }
    }
}

impl Disable {
    pub fn new(enable: bool, canons: HashMap<String, String>) -> Self {
        Disable { enable, canons }
    }
    
    pub fn set_enabled(&self, cmd: &str, ctx: &Context, db: &mut Db) -> Outcome<()> {
        if self.enable {
            db.silences.remove(&ctx.channel, &cmd);
            #[cfg(not(test))] diesel
                ::delete(silence::table
                    .filter(silence::channel.eq(&ctx.channel))
                    .filter(silence::command.eq(&cmd))
                ).execute(&db.conn)?;
        } else {
            let silence = db::Silence { channel: ctx.channel.to_owned(), command: cmd.to_owned() };
            #[cfg(not(test))] diesel
                ::insert_into(silence::table)
                .values(&silence)
                .on_conflict_do_nothing()
                .execute(&db.conn)?;
            db.silences.insert(silence);
        }
        Ok(())
    }
}
