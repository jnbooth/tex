use std::fmt::Debug;

pub use self::Level::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Level {
    ERROR,
    WARNING,
    INFO,

    ECHO,
    ASK
}

#[inline]
fn color(lvl: Level) -> u8 {
    match lvl {
        ERROR   => 31,
        WARNING => 33,
        INFO    => 34,
        ECHO    => 32,
        ASK     => 37
    }
}

#[inline]
fn label(lvl: Level) -> &'static str {
    match lvl {
        ERROR   => "ERROR: ",
        WARNING => "WARNING: ",
        INFO    => "INFO: ",
        _       => ""
    }
}

#[inline] 
fn clean(s: &str) -> String {
    s.replace('\x02',"").replace('\x1d', "")
}

#[inline]
pub fn log(lvl: Level, s: &str) {
    println!("\x1b[{}m{}{}\x1b[0m", color(lvl), label(lvl), clean(s));
}
#[inline]
pub fn log_part(lvl: Level, s: &str) {
    print!("\x1b[{}m{}{}\x1b[0m", color(lvl), label(lvl), clean(s));
}

pub trait Logged {
    fn log(self, label: &str);
}

impl<T, E: Debug> Logged for Result<T, E> {
    fn log(self, label: &str) {
        if let Err(e) = self {
            log(ERROR, &format!("{}: {:?}", label, e));
        }
    }
}

macro_rules! trace {
    () => {
        &format!("{}:{}:{}", file!(), line!(), column!())
    }
}
