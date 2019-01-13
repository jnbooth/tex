pub const DEBUG: u8 = 34;
pub const ECHO:  u8 = 32;
pub const WARN:  u8 = 33;
pub const ASK:   u8 = 37;

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
