use super::*;

pub struct Reload;

impl Command for Reload {
    fn cmds(&self) -> Vec<String> {
        own(&["reload"])
    }
    fn usage(&self) -> String { "".to_owned() }
    fn fits(&self, size: usize) -> bool { size == 0 }
    fn auth(&self) -> Auth { Owner }

    fn run(&mut self, _: &[&str], _: &Context, db: &mut Db) -> Outcome {
        db.reload().map_err(Throw)?;
        Ok(vec![Action("reloads its database.".to_owned())])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{Reminder, Silence, Tell};

    #[test] #[ignore]
    fn clears_silences() {
        let mut db = Db::default();
        db.silences.insert(Silence::default());
        Reload.run(&[], &Context::default(), &mut db).unwrap();
        assert!(db.silences.is_empty());
    }

    #[test] #[ignore]
    fn clears_reminders() {
        let mut db = Db::default();
        db.reminders.insert("".to_owned(), Reminder::default());
        Reload.run(&[], &Context::default(), &mut db).unwrap();
        assert!(db.reminders.is_empty());
    }

    #[test] #[ignore]
    fn clears_tells() {
        let mut db = Db::default();
        db.tells.insert("".to_owned(), Tell::default());
        Reload.run(&[], &Context::default(), &mut db).unwrap();
        assert!(db.tells.is_empty());
    }
}
