use regex::Regex;
use std::time::Duration;

fn yield_offset(d: u32, h: u32, m: u32) -> Option<Duration> {
    println!("{}d{}h{}m", d, h, m);
    Some(Duration::from_secs(60 * (m + 60 * (h + 24 * d)) as u64))
}

fn next<'r, 't>(groups: &mut regex::Matches<'r, 't>) -> Option<u32> {
    groups.next()?.as_str().parse().ok()
}

pub fn parse_offset(s: &str) -> Option<Duration> {
    lazy_static! {
        static ref RE: Regex = Regex::new("\\d+").unwrap();
    }
    let format: &str = &RE.replace_all(s, "*").into_owned();
    let mut groups = RE.find_iter(s);
    match format {
        "*d*h*m" => yield_offset(next(&mut groups)?, next(&mut groups)?, next(&mut groups)?),
        "*d*h"   => yield_offset(next(&mut groups)?, next(&mut groups)?, 0),
        "*d*m"   => yield_offset(next(&mut groups)?, 0,                  next(&mut groups)?),
        "*d"     => yield_offset(next(&mut groups)?, 0,                  0),
        "*h*m"   => yield_offset(0,                  next(&mut groups)?, next(&mut groups)?),
        "*h"     => yield_offset(0,                  next(&mut groups)?, 0),
        "*m"     => yield_offset(0,                  0,                  next(&mut groups)?),
        _        => None
    }
}
