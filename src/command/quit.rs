use super::*;

pub struct Quit;

impl<O: Output + 'static> Command<O> for Quit {
    fn cmds(&self) -> Vec<String> {
        own(&["quit"])
    }
    fn usage(&self) -> String { "".to_owned() }
    fn fits(&self, size: usize) -> bool { size == 0 }
    fn auth(&self) -> i32 { 3 }
    fn reload(&mut self, _: &mut Db) -> Outcome<()> { Ok(()) }

    fn run(&mut self, _: &[&str], irc: &O, _: &Context, _: &mut Db) -> Outcome<()> {
        irc.quit("Shutting down, bleep bloop.")?;
        Ok(())
    }
}
