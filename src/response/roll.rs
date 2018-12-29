use rand::Rng;
use rand::rngs::ThreadRng;

use super::super::IO;

pub fn throw(s: &str) -> IO<String> {
    let mut throw = s.to_owned();
    if !throw.starts_with('+') && !throw.starts_with('-') {
        throw = format!("+ {}", throw);
    }
    let dice: Vec<&str> = throw.split(' ').filter(|x| !x.is_empty()).collect();
    if dice.len() & 1 != 0 {
        return Err(failure::err_msg("Wrong number of arguments."));
    }
    let mut score: i64 = 0;
    let mut rng = rand::thread_rng();
    for chunk in dice.chunks_exact(2) {
        match chunk {
            &[sign, die] => {
                let signum: i64 = match sign {
                    "+" => 1,
                    "-" => -1,
                    _   => Err(failure::err_msg(format!("{} is neither '+' nor '-'.", sign)))?
                };
                match die.find('d') {
                    None => {
                        let bonus: u32 = die.parse()?;
                        score = score + signum * bonus as i64;
                    },
                    Some(i) => {
                        let (before, after_raw) = die.split_at(i);
                        let mut after = after_raw[1..].to_string();
                        let amount: u16 = if before.is_empty() { 1 } else { before.parse()? };
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
                            score += signum * max * amount as i64;
                        } else if min < max {
                            for _ in 0..amount {
                                score += signum * roll(&mut rng, min, max, explode, threshold, cmp);
                            }
                        }
                    }
                }
            },
            _ => return Err(failure::err_msg("Chunking error."))
        }
    }
    Ok(score.to_string())
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
