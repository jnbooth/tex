use super::*;

pub struct Reload;

impl<O: Output + 'static> Command<O> for Reload {
    fn cmds(&self) -> Vec<String> {
        own(&["reload"])
    }
    fn usage(&self) -> String { "".to_owned() }
    fn fits(&self, size: usize) -> bool { size == 0 }
    fn auth(&self) -> i32 { 3 }
    fn reload(&mut self, _: &mut Db) -> Outcome<()> { Ok(()) }

    fn run(&mut self, _: &[&str], irc: &O, ctx: &Context, db: &mut Db) -> Outcome<()> {
        db.reload()?;
        Ok(irc.action(ctx, "reloads its database.")?)
    }
}