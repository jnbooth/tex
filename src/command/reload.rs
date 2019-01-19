use super::*;

pub struct Reload;

impl Command for Reload {
    fn cmds(&self) -> Vec<String> {
        own(&["reload"])
    }
    fn usage(&self) -> String { "".to_owned() }
    fn fits(&self, size: usize) -> bool { size == 0 }
    fn auth(&self) -> i32 { 3 }

    fn run(&mut self, _: &[&str], _: &Context, db: &mut Db) -> Outcome {
        db.reload()?;
        Ok(vec![Action("reloads its database.".to_owned())])
    }
}
