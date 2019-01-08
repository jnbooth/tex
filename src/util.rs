use core::hash::Hash;
use multimap::MultiMap;
use percent_encoding::utf8_percent_encode;
use std::iter::*;
use std::time::{Duration, SystemTime, SystemTimeError};

pub fn encode(s: &str) -> String {
    utf8_percent_encode(s, percent_encoding::DEFAULT_ENCODE_SET).to_string()
}

pub fn since(when: SystemTime) -> Result<String, SystemTimeError> {
    let dur = when.elapsed()?.as_secs();
    Ok(humantime::format_duration(
        Duration::from_secs(if dur < 60 { dur } else { dur / 60 * 60 })
    ).to_string())
}

pub fn show_time(when: SystemTime) -> String {
    let time = humantime::format_rfc3339_seconds(
        when - Duration::from_secs(60 * 60 * 8)
    ).to_string();
    time[..time.len()-4].rsplit('T').collect::<Vec<&str>>().join(" ").replace("-", "/")
}

pub fn split_on<'a>(pat: &str, s: &'a str) -> Option<(&'a str, &'a str)> {
    match s.find(pat) {
        None => None,
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
