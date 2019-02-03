use chrono::{DateTime, Duration, NaiveDate, Utc};
use multimap::MultiMap;
use percent_encoding::utf8_percent_encode;
use std::hash::Hash;
use std::iter::*;
use std::string::ToString;
use std::time::SystemTime;

const CHARACTER_LIMIT: usize = 300;

pub fn encode(s: &str) -> String {
    utf8_percent_encode(s, percent_encoding::DEFAULT_ENCODE_SET).to_string()
}

pub fn own(xs: &[&str]) -> Vec<String> {
    xs.into_iter().map(ToString::to_string).collect()
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

pub fn rating(i: i64) -> String {
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

#[inline]
fn show_ago(amount: i64, s: &str) -> String {
    match amount {
        1 => format!("{} {}", amount, s),
        _ => format!("{} {}s", amount, s)
    }
}

pub fn ago<T: DurationAgo>(time: T) -> String {
    let dur = time.duration_ago();
    if dur.num_days() > 365 {
        show_ago(dur.num_days() / 365, "year")
    } else if dur.num_weeks() > 0 {
        show_ago(dur.num_weeks(), "week")
    } else if dur.num_days() > 0 {
        show_ago(dur.num_days(), "day")
    } else if dur.num_hours() > 0 {
        show_ago(dur.num_hours(), "hour")
    } else if dur.num_minutes() > 0 {
        show_ago(dur.num_minutes(), "minute")
    } else if dur.num_seconds() > 0 {
        show_ago(dur.num_seconds(), "second")
    } else {
        "a few seconds".to_owned()
    }
}

pub fn show_time(time: SystemTime) -> String {
    let str = humantime::format_rfc3339_seconds(
        time - std::time::Duration::from_secs(60 * 60 * 8)
    ).to_string();
    str[..str.len()-4].rsplit('T').collect::<Vec<&str>>().join(" ").replace("-", "/")
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


#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Gender {
    Any,
    Female,
    Male
}
