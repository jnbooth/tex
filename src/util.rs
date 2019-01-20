use chrono::{DateTime, Duration, NaiveDate, Utc};
use percent_encoding::utf8_percent_encode;
use std::iter::*;
use std::string::ToString;
use std::time::SystemTime;

const CHARACTER_LIMIT: usize = 300;

pub fn encode(s: &str) -> String {
    utf8_percent_encode(s, percent_encoding::DEFAULT_ENCODE_SET).to_string()
}

pub fn own(xs: &[&str]) -> Vec<String> {
    xs.into_iter()
        .map(ToString::to_string)
        .collect()
}

pub fn trim(s: &str) -> String {
    let mut content = s.to_owned();
    if content.len() > CHARACTER_LIMIT {
        if let Some(i) = content[..CHARACTER_LIMIT-4].rfind(' ') {
            content = content[..i].to_owned();
        }
        content.push_str(" […]");
    }
    content
}

pub fn rating(i: i32) -> String {
    if i > 0 {
        format!("+{}", i)
    } else if i < 0 {
        format!("-{}", i)
    } else {
        format!("{}", i)
    }
}

pub fn parse_date(s: &str) -> Option<DateTime<Utc>> {
    let mut fragments = if s.contains('-') {
        s.split('-')
    } else {
        s.split('/')
    }.rev();
    let year = fragments.next()?.parse().ok()?;
    let month = if let Some(frag) = fragments.next() {
        frag.parse().ok()?
    } else {
        0
    };
    let day = if let Some(frag) = fragments.next() {
        frag.parse().ok()?
    } else {
        0
    };
    let naive = NaiveDate::from_ymd_opt(year, month, day)?.and_hms(0, 0, 0);
    Some(DateTime::from_utc(naive, Utc))
}

pub trait DurationAgo {
    fn duration_ago(self) -> Duration;
}
impl DurationAgo for SystemTime {
    fn duration_ago(self) -> Duration {
        match self.elapsed() {
            Err(_)  => Duration::zero(),
            Ok(dur) => Duration::seconds(dur.as_secs() as i64)
        }
    }
}
impl DurationAgo for DateTime<Utc> {
    fn duration_ago(self) -> Duration {
        Utc::now().signed_duration_since(self)
    }
}

pub fn ago<T: DurationAgo>(when: T) -> String {
    let dur = when.duration_ago();
    if dur.num_days() > 365 {
        format!("{} years", dur.num_days() / 365)
    } else if dur.num_weeks() > 1 {
        format!("{} weeks", dur.num_weeks())
    } else if dur.num_days() > 1 {
        format!("{} days", dur.num_days())
    } else if dur.num_hours() > 1 {
        format!("{} hours", dur.num_hours())
    } else if dur.num_minutes() > 1 {
        format!("{} minutes", dur.num_minutes())
    } else if dur.num_seconds() > 1 {
        format!("{} seconds", dur.num_seconds())
    } else {
        "a few seconds".to_owned()
    }
}

pub fn show_time(when: SystemTime) -> String {
    let time = humantime::format_rfc3339_seconds(
        when - std::time::Duration::from_secs(60 * 60 * 8)
    ).to_string();
    time[..time.len()-4].rsplit('T').collect::<Vec<&str>>().join(" ").replace("-", "/")
}

pub fn split_on<'a>(pat: &str, s: &'a str) -> Option<(&'a str, &'a str)> {
    match s.find(pat) {
        None    => None,
        Some(i) => {
            let (before, after) = s.split_at(i);
            Some((before, &after[pat.len()..]))
        }
    }
}


pub fn drain_filter<F, T>(vec: &mut Vec<T>, filter: F) -> Vec<T> where F: Fn(&T) -> bool {
    let mut drained = Vec::new();
    let mut i = 0;
    while i != vec.len() {
        if filter(&vec[i]) {
            drained.push(vec.remove(i));
        } else {
            i += 1;
        }
    }
    drained
}

pub fn pop_filter<F, T>(vec: &mut Vec<T>, filter: F) -> Option<T> where F: Fn(&T) -> bool {
    for i in 0..vec.len() {
        if filter(&vec[i]) {
            return Some(vec.remove(i))
        }
    }
    None
}

/*
pub fn multi_remove<K: Eq + Hash, V: Eq>(map: &mut MultiMap<K, V>, k: &K, v: &V) -> bool {
    if let Some(vec) = map.get_vec_mut(k) {
        let mut i = 0;
        while i != vec.len() {
            if &vec[i] == v {
                vec.remove(i);
                return true
            } else {
                i += 1;
            }
        }
    }
    false
}

pub fn since(when: SystemTime) -> Result<String, SystemTimeError> {
    let secs = when.elapsed()?.as_secs();
    Ok(humantime::format_duration(
        Duration::from_secs(if secs < 60 { secs } else { secs / 60 * 60 })
    ).to_string())
}
*/


#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Gender {
    Any,
    Female,
    Male
}
