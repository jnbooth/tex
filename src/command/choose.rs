use rand::Rng;
use rand::rngs::ThreadRng;

use super::*;

pub struct Choose {
    rng: ThreadRng
}

impl<O: Output + 'static> Command<O> for Choose {
    fn cmds(&self) -> Vec<String> {
        abbrev("choose")
    }
    fn usage(&self) -> String { "<choices, separated, by, commas>".to_owned() }
    fn fits(&self, size: usize) -> bool { size > 0 }
    fn auth(&self) -> i32 { 0 }
    fn reload(&mut self, _: &mut Db) -> Outcome<()> { Ok(()) }

    fn run(&mut self, args: &[&str], irc: &O, ctx: &Context, _: &mut Db) -> Outcome<()> {
        let choices = args.join(" ");
        let opts: Vec<&str> = choices.split(',').map(str::trim).collect();
        Ok(irc.reply(ctx, self.choose(opts))?)
    }
}

impl Choose {
    pub fn new() -> Self {
        Choose { rng: rand::thread_rng() }
    }
    fn choose<'a>(&mut self, xs: Vec<&'a str>) -> &'a str {
        xs[self.rng.gen_range(0, xs.len())]
    }
}
