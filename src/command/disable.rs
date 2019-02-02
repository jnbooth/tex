use super::*;
use crate::db::{Silence, silence};

pub struct Disable {
    enable: bool,
    canons: HashMap<String, String>
}

impl Command for Disable {
    fn cmds(&self) -> Vec<String> {
        if self.enable { own(&["enable"]) } else { own(&["disable"]) }
    }
    fn usage(&self) -> String { "<command>".to_owned() }
    fn fits(&self, size: usize) -> bool { size == 1 }
    fn auth(&self) -> u8 { 2 }

    fn run(&mut self, args: &[&str], ctx: &Context, db: &mut Db) -> Outcome {
        let cmd = args.join(" ").to_lowercase();
        let canon = match cmd.chars().next() {
            Some('!') => self.canons.get(&cmd[1..]),
            Some('.') => self.canons.get(&cmd[1..]),
            _         => self.canons.get(&cmd)
        };
        match canon {
            None        => Ok(vec![Reply("I'm sorry, I don't know that command.".to_owned())]),
            Some(canon) => {
                self.set_enabled(&canon, ctx, db)?;
                Ok(vec![Action(
                    format!("{}s .{}.", if self.enable { "enable" } else { "disable" }, canon)
                )])
            }
        }
    }
}

impl Disable {
    pub fn new(enable: bool, canons: HashMap<String, String>) -> Self {
        Self { enable, canons }
    }
    
    pub fn set_enabled(&self, cmd: &str, ctx: &Context, db: &mut Db) -> Result<(), Error> {
        let conn = db.conn();
        if self.enable {
            db.silences.remove(&ctx.channel, &cmd);
            diesel::delete(silence::table
                .filter(silence::channel.eq(&ctx.channel))
                .filter(silence::command.eq(&cmd))
            ).execute(&conn)?;
        } else {
            let silence = Silence { channel: ctx.channel.to_owned(), command: cmd.to_owned() };
            diesel::insert_into(silence::table)
                .values(&silence)
                .on_conflict_do_nothing()
            .execute(&conn)?;
            db.silences.insert(silence);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    const CMD: &str = "x";

    fn new(enable: bool) -> Disable {
        let mut canons = HashMap::new();
        canons.insert(CMD.to_owned(), CMD.to_owned());
        Disable::new(enable, canons)
    }
    fn is_enabled(cmd: &str, db: &Db) -> bool {
        !db.silences.contains(&Context::default().channel, cmd)
    }

    #[test] #[ignore]
    fn disables() {
        let mut db = Db::default();
        let mut enable = new(true);
        enable.run(&[CMD], &Context::default(), &mut db).unwrap();
        let mut disable = new(false);
        disable.run(&[CMD], &Context::default(), &mut db).unwrap();
        assert!(!is_enabled(CMD, &db));
    }

    #[test] #[ignore]
    fn enables() {
        let mut db = Db::default();
        let mut disable = new(false);
        disable.run(&[CMD], &Context::default(), &mut db).unwrap();
        let mut enable = new(true);
        enable.run(&[CMD], &Context::default(), &mut db).unwrap();
        assert!(is_enabled(CMD, &db));
    }

    #[test]
    fn not_found() {
        let mut db = Db::default();
        let mut disable = new(false);
        disable.run(&["y"], &Context::default(), &mut db).unwrap();
        assert!(is_enabled("y", &db));
    }
}
