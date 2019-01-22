use super::*;
use crate::util;

pub struct Seen;

impl Command for Seen {
    fn cmds(&self) -> Vec<String> {
        own(&["seen", "se"])
    }
    fn usage(&self) -> String { "[#<channel>] [-f|-t] <user>".to_owned() }
    fn fits(&self, size: usize) -> bool { size >= 1 }
    fn auth(&self) -> i32 { 0 }

    fn run(&mut self, args: &[&str], ctx: &Context, db: &mut Db) -> Outcome {
        Ok(vec![Reply(search(args, ctx,  db)?)])
    }
}


#[derive(PartialEq)]
pub enum Mode {
    First,
    Regular,
    Total
}

pub fn mode(s: &str) -> Option<Mode> {
    match s {
        "-f"      => Some(Mode::First),
        "-t"      => Some(Mode::Total),
        "--first" => Some(Mode::First),
        "--total" => Some(Mode::Total),
        _         => None
    }
}

fn search(args_im: &[&str], ctx: &Context, db: &Db) -> Result<String, Error> {
    let mut args = args_im.to_owned();
    let mode = match util::pop_filter(&mut args, |x| x.starts_with('-')) {
        None       => Mode::Regular,
        Some(flag) => mode(flag).ok_or(InvalidArgs)?
    };
    let channel = util::pop_filter(&mut args, |x| x.starts_with('#'))
        .map(ToOwned::to_owned)
        .unwrap_or_else(||ctx.channel.to_owned());
    match args.as_slice() {
        [nick] => find(&nick, &channel, &mode, db).ok_or(NoResults),
        _      => Err(InvalidArgs)
    }
}

fn find(nick: &str, channel: &str, mode: &Mode, db: &Db) -> Option<String> {
    let seen = db.get_seen(channel, nick).ok()?;
    match mode {
        Mode::First => Some(format!(
            "I first saw \x02{}\x02 {} ago, saying: {}", 
            nick, util::ago(seen.first_time), seen.first
        )),
        Mode::Regular => Some(format!(
            "I last saw \x02{}\x02 {} ago, saying: {}",
            nick, util::ago(seen.latest_time), seen.latest
        )),
        Mode::Total => Some(format!(
            "I have seen \x02{}\x02 total message{} from \x02{}\x02.",
            seen.total, if seen.total != 1 { "s" } else { "" }, nick
        ))
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    fn ctx_test() -> Context {
        Context::mock("#@", "@A")
    }

    fn db_test() -> Db {
        let ctx = ctx_test();
        let mut db = Db::default();
        db.add_seen(&ctx, "first").expect("Error adding first");
        db.add_seen(&ctx, "latest").expect("Error adding last");
        db
    }

    #[test]
    fn first() {
        let ctx = ctx_test();
        assert_eq!(
            search(&["-f", &ctx.nick], &ctx, &db_test()).ok().unwrap(), 
            format!("I first saw \x02{}\x02 a few seconds ago, saying: first", ctx.nick)
        );
    }

    #[test]
    fn latest() {
        let ctx = ctx_test();
        assert_eq!(
            search(&[&ctx.nick], &ctx, &db_test()).ok().unwrap(), 
            format!("I last saw \x02{}\x02 a few seconds ago, saying: latest", ctx.nick)
        );
    }

    #[test]
    fn total() {
        let ctx = ctx_test();
        assert_eq!(
            search(&["-t", &ctx.nick], &ctx, &db_test()).ok().unwrap(), 
            format!("I have seen \x022\x02 total messages from \x02{}\x02.", ctx.nick)
        );
    }

    #[test]
    fn compound() {
        let ctx = ctx_test();
        let fake = Context::mock("#!!", &ctx.user);
        assert_eq!(
            search(&[&ctx.nick, "-t", &ctx.channel], &fake, &db_test()).ok().unwrap(), 
            format!("I have seen \x022\x02 total messages from \x02{}\x02.", ctx.nick)
        );
    }


    #[test]
    fn privmsg_is_none() {
        let ctx = Context::mock("@A", "@A");
        let mut db = Db::default();
        db.add_seen(&ctx, "!").ok().unwrap();
        assert!(search(&[&ctx.nick], &ctx, &db_test()).is_err());
    }

    #[test]
    fn unseen_is_none() {
        let ctx = Context::mock("@A", "#@");
        assert!(search(&[&ctx.nick], &ctx, &Db::default()).is_err());
    }

    #[test]
    fn different_channel_is_none() {
        let ctx = Context::mock("@A", "#@");
        let fake = Context::mock("#!!", &ctx.user);
        assert!(search(&[&ctx.nick, "-t"], &fake, &db_test()).is_err());
    }
}
