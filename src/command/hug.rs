use super::*;

pub struct Hug;

impl<O: Output + 'static> Command<O> for Hug {
    fn cmds(&self) -> Vec<String> {
        own(&["hug", "hugs", "hugme"])
    }
    fn usage(&self) -> String { "".to_owned() }
    fn fits(&self, size: usize) -> bool { size == 0 }
    fn auth(&self) -> i32 { 0 }
    fn reload(&mut self, _: &mut Db) -> Outcome<()> { Ok(()) }

    fn run(&mut self, _: &[&str], irc: &O, ctx: &Context, _: &mut Db) -> Outcome<()> {
        Ok(irc.action(ctx, &format!("hugs {}.", ctx.nick))?)
    }
}
