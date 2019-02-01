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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clears_loaded() {
        let mut db = Db::default();
        db.loaded.insert("x".to_owned());
        Reload.run(&[], &Context::default(), &mut db).unwrap();
        assert!(db.loaded.is_empty());
    }

    #[test]
    fn clears_silences() {
        let mut db = Db::default();
        db.silences.insert(db::Silence::default());
        Reload.run(&[], &Context::default(), &mut db).unwrap();
        assert!(db.silences.is_empty());
    }

    #[test]
    fn clears_reminders() {
        let mut db = Db::default();
        db.reminders.insert("".to_owned(), db::Reminder::default());
        Reload.run(&[], &Context::default(), &mut db).unwrap();
        assert!(db.reminders.is_empty());
    }

    #[test]
    fn clears_tells() {
        let mut db = Db::default();
        db.tells.insert("".to_owned(), db::Tell::default());
        Reload.run(&[], &Context::default(), &mut db).unwrap();
        assert!(db.tells.is_empty());
    }
}
