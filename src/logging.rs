use std::fmt::Debug;

pub const INFO: u8 = 34;
pub const WARN: u8 = 33;

pub const ECHO: u8 = 32;
pub const ASK:  u8 = 37;

#[inline] 
fn clean(s: &str) -> String {
    s.replace('\x02',"").replace('\x1d', "")
}

#[inline]
pub fn log(code: u8, s: &str) {
    println!("\x1b[{}m{}\x1b[0m", code, clean(s));
}
#[inline]
pub fn log_part(code: u8, s: &str) {
    print!("\x1b[{}m{}\x1b[0m", code, clean(s));
}

pub trait Logged {
    fn log(self, label: &str);
}

impl<T, E: Debug> Logged for Result<T, E> {
    fn log(self, label: &str) {
        if let Err(e) = self {
            println!("\x1b[{}m{}: {:?}\x1b[0m", WARN, label, e);
        }
    }
}

macro_rules! trace {
    () => {
        &format!("{}:{}:{}", file!(), line!(), column!())
    }
}
