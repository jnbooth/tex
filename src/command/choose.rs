use rand::Rng;
use rand::rngs::ThreadRng;

use super::*;

#[derive(Default)]
pub struct Choose {
    rng: ThreadRng
}

impl Command for Choose {
    fn cmds(&self) -> Vec<String> {
        abbrev("choose")
    }
    fn usage(&self) -> String { "<choices, separated, by, commas>".to_owned() }
    fn fits(&self, size: usize) -> bool { size >= 1 }
    fn auth(&self) -> u8 { 0 }

    fn run(&mut self, args: &[&str], _: &Context, _: &mut Db) -> Outcome {
        let choices = args.join(" ");
        let opts: Vec<&str> = choices.split(',').map(str::trim).collect();
        Ok(vec![Reply(opts[self.rng.gen_range(0, opts.len())].to_owned())])
    }
}

impl Choose { pub fn new() -> Self { Self::default() } }

#[cfg(test)]
mod tests {
    use super::*;
    use hashbrown::HashSet;

    #[test]
    fn chooses_any_option() {
        let mut db = Db::default();
        let ctx = Context::default();
        let mut choose = Choose::default();
        let choices = ["a", "b c", "d"];
        let args = choices.join(",");
        let set: HashSet<String> = own(&choices).into_iter().collect();
        let results: HashSet<String> = (0..crate::FUZZ)
            .map(|_| choose.test(&args, &ctx, &mut db).expect("Error running command."))
            .collect();
        assert_eq!(set, results);
    }
}
