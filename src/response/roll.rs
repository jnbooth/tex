use lazy_static::lazy_static;
use rand::Rng;
use regex::Regex;

use crate::{IO, util};

pub fn throw(s: &str) -> IO<i64> {
    lazy_static! {
        static ref RE_DICE: Regex = Regex::new("\\s*(\\+|-)\\s*")
            .expect("RE_DICE Regex failed to compile");
    }
    let mut score: i64 = 0;
    let mut rng = rand::thread_rng();
    for die in RE_DICE.replace_all(s, " $1").split(' ').filter(|x| !x.is_empty()) {
        match util::split_on(die, "d") {
            None => {
                score += die.parse::<i64>()?;
            },
            Some((before, after)) => {
                let amount: i16 = if before.is_empty() { 1 } else { before.parse()? };
                let mut suffix = after.to_owned();
                let signum = amount.signum() as i64;
                let (cmp, threshold) = if let Some(i) = suffix.find('>') {
                    (1, suffix.split_off(i)[1..].parse()?)
                } else if let Some(i) = suffix.find('<') {
                    (-1, suffix.split_off(i)[1..].parse()?)
                } else {
                    (0, 0)
                };
                let explode = if suffix.ends_with("!") {
                    suffix.pop();
                    true
                } else {
                    false
                };
                
                let (min, max): (i64, i64) = match suffix.as_ref() {
                    "f" => (-1, 1),
                    "F" => (-1, 1),
                    "%" => (1, 100),
                    _   => (1, suffix.parse()?)
                };
                if min == max {
                    score += amount as i64;
                } else if min < max {
                    for _ in 0..amount.abs() {
                        score += signum * roll(&mut rng, min, max, explode, cmp, threshold);
                    }
                }
            }
        }
    }
    Ok(score)
}

fn roll<T: Rng>(rng: &mut T, min: i64, max: i64, explode: bool, cmp: i64, threshold: i64) -> i64 {
    let side = rng.gen_range(min, max + 1);
    let score: i64 = if cmp > 0 {
        if side > threshold { 1 } else { 0 }
    } else if cmp < 0 {
        if side < threshold { 1 } else { 0 }
    } else {
        side
    };
    
    if explode && side == max {
        score + roll(rng, min, max, explode, cmp, threshold)
    } else {
        score
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FUZZ: u16 = 100;

    #[test]
    fn test_roll() {
    let mut rng = rand::thread_rng();

        for _ in 0..FUZZ {
            let x = rng.gen();
            let y = rng.gen();
            let (min, max) = if x <= y { (x, y) } else { (y, x) };

            let result = roll(&mut rng, min, max, false, 0, 0);

            assert!(result >= min);
            assert!(result <= max);
        }
    }
}
