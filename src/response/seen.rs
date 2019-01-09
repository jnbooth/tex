use crate::db::Db;
use crate::util;

pub enum Error {
    InvalidArgs,
    NotFound
}

#[derive(PartialEq)]
pub enum Mode {
    First,
    Regular,
    Total
}

pub fn mode(s: &str) -> Option<Mode> {
    match s {
        "-f" => Some(Mode::First),
        "-t" => Some(Mode::Total),
        "--first" => Some(Mode::First),
        "--total" => Some(Mode::Total),
        _    => None
    }
}

pub fn search(db: &Db, target: &str, args_im: &[&str]) -> Result<String, Error> {
    let mut args = args_im.to_owned();
    let mode = match util::pop_filter(&mut args, |x| x.starts_with("-")) {
        None       => Mode::Regular,
        Some(flag) => mode(flag).ok_or(Error::InvalidArgs)?
    };
    let channel = util::pop_filter(&mut args, |x| x.starts_with("#"))
        .unwrap_or(target);
    match args.as_slice() {
        [nick] => find(db, channel, nick, mode).ok_or(Error::NotFound),
        _      => Err(Error::InvalidArgs)
    }
}

fn find(db: &Db, channel: &str, nick: &str, mode: Mode) -> Option<String> {
    let seen = db.get_seen(channel, nick).ok()??;
    match mode {
        Mode::First => Some(format!(
            "I first saw {} {} ago, saying: {}", 
            nick, util::since(seen.first_time).ok()?, seen.first
        )),
        Mode::Regular => Some(format!(
            "I last saw {} {} ago, saying: {}",
            nick, util::since(seen.latest_time).ok()?, seen.latest
        )),
        Mode::Total => Some(format!(
            "I have seen {} total message{} from {}.",
            seen.total, if seen.total != 1 { "s" } else { "" }, nick
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const USER: &str = "@";
    const CHAN: &str = "#@";

    fn db_test() -> Db {
        let mut db = Db::new();
        db.add_seen(CHAN, USER, "first").expect("Error adding first");
        db.add_seen(CHAN, USER, "latest").expect("Error adding last");
        db
    }

    #[test]
    fn test_first() {
        assert_eq!(
            search(&db_test(), CHAN, &["-f", USER]).ok().unwrap(), 
            format!("I first saw {} 0s ago, saying: first", USER)
        );
    }

    #[test]
    fn test_latest() {
        assert_eq!(
            search(&db_test(), CHAN, &[USER]).ok().unwrap(), 
            format!("I last saw {} 0s ago, saying: latest", USER)
        );
    }

    #[test]
    fn test_total() {
        assert_eq!(
            search(&db_test(), CHAN, &["-t", USER]).ok().unwrap(), 
            format!("I have seen 2 total messages from {}.", USER)
        );
    }

    #[test]
    fn test_compound() {
        assert_eq!(
            search(&db_test(), "#!", &[USER, "-t", CHAN]).ok().unwrap(), 
            format!("I have seen 2 total messages from {}.", USER)
        );
    }


    #[test]
    fn test_privmsg_is_none() {
        let mut db = Db::new();
        db.add_seen(USER, USER, "!").ok().unwrap();
        assert!(search(&db, USER, &[USER]).is_err());
    }

    #[test]
    fn test_unseen_is_none() {
        assert!(search(&Db::new(), CHAN, &[USER]).is_err());
    }
}
