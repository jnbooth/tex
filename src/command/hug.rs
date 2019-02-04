use super::*;

pub struct Hug;

impl Command for Hug {
    fn cmds(&self) -> Vec<String> {
        own(&["hug", "hugs", "hugme"])
    }
    fn usage(&self) -> String { "".to_owned() }
    fn fits(&self, size: usize) -> bool { size == 0 }
    fn auth(&self) -> Auth { Anyone }

    fn run(&mut self, _: &[&str], ctx: &Context, _: &mut Db) -> Outcome {
        Ok(vec![Action(format!("hugs {}.", ctx.nick))])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hugs() {
        assert_eq!(Hug.test_def("").unwrap(), "hugs .");
    }
}
