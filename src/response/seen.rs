use std::time::{Duration, SystemTime, SystemTimeError};
use super::super::db::Db;

#[derive(PartialEq)]
pub enum Mode {
    First,
    Invalid,
    Regular,
    Total
}

pub fn mode(s: &str) -> Mode {
    match s.get(1..) {
        Some("f") => Mode::First,
        Some("t") => Mode::Total,
        _         => Mode::Invalid
    }
}

fn since(when: SystemTime) -> Result<String, SystemTimeError> {
    let dur = when.elapsed()?.as_secs();
    Ok(humantime::format_duration(
        Duration::from_secs(if dur < 60 { dur } else { dur / 60 * 60 })
    ).to_string())
}

pub fn search(db: &Db, channel_up: &str, nick_up: &str, mode: Mode) -> Option<String> {
    let seen = db.get_seen(channel_up, nick_up)?;
    match mode {
        Mode::First => Some(format!(
            "I first saw {} {} ago, saying: {}", 
            nick_up, since(seen.first_time).ok()?, seen.first
        )),
        Mode::Regular => Some(format!(
            "I last saw {} {} ago, saying: {}",
            nick_up, since(seen.latest_time).ok()?, seen.latest
        )),
        Mode::Invalid => None,
        Mode::Total => Some(format!(
            "I have seen {} total message{} from {}.",
            seen.total, if seen.total != 1 { "s" } else { "" }, nick_up
        ))
    }
}
