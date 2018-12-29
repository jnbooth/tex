use std::time::{Duration, SystemTime, SystemTimeError};
use super::super::db::Db;
use super::super::vec::pop_filter;

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

pub fn search(db: &Db, target: &str, args_im: &Vec<&str>) -> Result<String, Error> {
    let mut args = args_im.clone();
    let mode = match pop_filter(&mut args, |x| x.starts_with("-")) {
        None       => Mode::Regular,
        Some(flag) => mode(flag).ok_or(Error::InvalidArgs)?
    };
    let channel = pop_filter(&mut args, |x| x.starts_with("#"))
        .unwrap_or(target);
    match args.as_slice() {
        [nick] => find(db, channel, nick, mode).ok_or(Error::NotFound),
        _      => Err(Error::InvalidArgs)
    }
}

fn since(when: SystemTime) -> Result<String, SystemTimeError> {
    let dur = when.elapsed()?.as_secs();
    Ok(humantime::format_duration(
        Duration::from_secs(if dur < 60 { dur } else { dur / 60 * 60 })
    ).to_string())
}

fn find(db: &Db, channel: &str, nick: &str, mode: Mode) -> Option<String> {
    let seen = db.get_seen(channel, nick)?;
    match mode {
        Mode::First => Some(format!(
            "I first saw {} {} ago, saying: {}", 
            nick, since(seen.first_time).ok()?, seen.first
        )),
        Mode::Regular => Some(format!(
            "I last saw {} {} ago, saying: {}",
            nick, since(seen.latest_time).ok()?, seen.latest
        )),
        Mode::Total => Some(format!(
            "I have seen {} total message{} from {}.",
            seen.total, if seen.total != 1 { "s" } else { "" }, nick
        ))
    }
}
