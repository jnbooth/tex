use super::*;

pub struct Zyn;

impl Command for Zyn {
    fn cmds(&self) -> Vec<String> {
        abbrev("zyn")
    }
    fn usage(&self) -> String { "".to_owned() }
    fn fits(&self, size: usize) -> bool { size == 0 }
    fn auth(&self) -> i32 { 0 }

    fn run(&mut self, _: &[&str], _: &Context, _: &mut Db) -> Outcome {
        Ok(vec![Reply("Marp.".to_owned())])
    }
}
