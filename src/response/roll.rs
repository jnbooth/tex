use rand::Rng;
use rand::rngs::ThreadRng;
use regex::Regex;

use super::super::IO;

pub fn throw(s: &str) -> IO<String> {
    lazy_static! {
        static ref RE: Regex = Regex::new("\\s*(\\+|-)\\s*").unwrap();
    }
    let mut score: i64 = 0;
    let mut rng = rand::thread_rng();
    for die in RE.replace_all(s, " $1").split(' ').filter(|x| !x.is_empty()) {
        match die.find('d') {
            None => {
                let bonus: i64 = die.parse()?;
                score = score + bonus;
            },
            Some(i) => {
                let (before, after_raw) = die.split_at(i);
                let mut after = after_raw[1..].to_string();
                let amount: i16 = if before.is_empty() { 1 } else { before.parse()? };
                let signum = amount.signum() as i64;
                let (cmp, threshold) = if let Some(i) = after.find('>') {
                    (1, after.split_off(i)[1..].parse()?)
                } else if let Some(i) = after.find('<') {
                    (-1, after.split_off(i)[1..].parse()?)
                } else {
                    (0, 0)
                };
                let explode = if after.ends_with("!") {
                    after = after[..after.len()-1].to_string();
                    true
                } else {
                    false
                };
                
                let (min, max): (i64, i64) = match after.as_ref() {
                    "f" => (-1, 1),
                    "F" => (-1, 1),
                    "%" => (1, 100),
                    _   => (1, after.parse()?)
                };
                if min == max {
                    score += amount as i64;
                } else if min < max {
                    for _ in 0..amount.abs() {
                        score += signum * roll(&mut rng, min, max, explode, threshold, cmp);
                    }
                }
            }
        }
    }
    Ok(format!("{} (rolled {})", score, s.replace(" ", "")))
}

fn roll(rng: &mut ThreadRng, min: i64, max: i64, explode: bool, threshold: i64, cmp: i64) -> i64 {
    let side = rng.gen_range(min, max + 1);
    let score: i64 = if cmp > 0 {
        if side > threshold { 1 } else { 0 }
    } else if cmp < 0 {
        if side < threshold { 1 } else { 0 }
    } else {
        side
    };
    
    if explode && side == max {
        score + roll(rng, min, max, explode, threshold, cmp)
    } else {
        score
    }
}
