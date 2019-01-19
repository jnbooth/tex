use super::*;

pub struct Hug;

impl Command for Hug {
    fn cmds(&self) -> Vec<String> {
        own(&["hug", "hugs", "hugme"])
    }
    fn usage(&self) -> String { "".to_owned() }
    fn fits(&self, size: usize) -> bool { size == 0 }
    fn auth(&self) -> i32 { 0 }

    fn run(&mut self, _: &[&str], ctx: &Context, _: &mut Db) -> Outcome {
        Ok(vec![Reply(format!("hugs {}.", ctx.nick))])
    }
}
