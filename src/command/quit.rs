use super::*;

pub struct Quit;

impl Command for Quit {
    fn cmds(&self) -> Vec<String> {
        own(&["quit"])
    }
    fn usage(&self) -> String { "".to_owned() }
    fn fits(&self, size: usize) -> bool { size == 0 }
    fn auth(&self) -> i32 { 3 }

    fn run(&mut self, _: &[&str], _: &Context, _: &mut Db) -> Outcome {
        Ok(vec![Response::Quit("Shutting down, bleep bloop.".to_owned())])
    }
}
