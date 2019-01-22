use rand::Rng;
use rand::rngs::ThreadRng;
use regex::Regex;

use super::*;
use crate::util;

pub struct Roll {
    dice: Regex,
    rng:  ThreadRng
}

impl Command for Roll {
    fn cmds(&self) -> Vec<String> {
        own(&["roll", "throw"])
    }
    fn usage(&self) -> String { 
        "<dice>. Examples: [\x02roll\x02 d20 + 4 - 2d6!], [\x02roll\x02 3dF-2], [\x02roll\x02 2d6>3 - 1d4].".to_owned() 
    }
    fn fits(&self, size: usize) -> bool { size >= 1 }
    fn auth(&self) -> i32 { 0 }

    fn run(&mut self, args: &[&str], _: &Context, _: &mut Db) -> Outcome {
        let content = args.join(" ");
        match self.throw(&content) {
            Err(NoResults) => Err(InvalidArgs),
            Err(err)       => Err(err),
            Ok(roll)       => Ok(vec![Reply(format!("\x02{}\x02 (rolled {})", roll, content))])
        }
    }
}

impl Default for Roll { fn default() -> Self { Self::new() } }

impl Roll {
    pub fn new() -> Self {
        Self { 
            dice: Regex::new("\\s*([+-])\\s*").expect("Dice regex failed to compile"),
            rng:  rand::thread_rng() 
        }
    }
    
    fn roll(&mut self, min: i64, max: i64, explode: bool, cmp: i64, threshold: i64) -> i64 {
        let side = self.rng.gen_range(min, max + 1);
        let score: i64 = if cmp > 0 {
            if side > threshold { 1 } else { 0 }
        } else if cmp < 0 {
            if side < threshold { 1 } else { 0 }
        } else {
            side
        };
        
        if explode && side == max {
            score + self.roll(min, max, explode, cmp, threshold)
        } else {
            score
        }
    }
    
    fn throw(&mut self, s: &str) -> Result<i64, Error> {
        let mut score: i64 = 0;
        for die in self.dice.replace_all(s, " $1").split(' ').filter(|x| !x.is_empty()) {
            match util::split_on("d", die) {
                None                  => score += die.parse::<i64>()?,
                Some((before, after)) => {
                    let amount: i16 = if before.is_empty() { 1 } else { before.parse()? };
                    let mut suffix = after.to_owned();
                    let signum = i64::from(amount.signum());
                    let (cmp, threshold) = if let Some(i) = suffix.find('>') {
                        (1, suffix.split_off(i)[1..].parse()?)
                    } else if let Some(i) = suffix.find('<') {
                        (-1, suffix.split_off(i)[1..].parse()?)
                    } else {
                        (0, 0)
                    };
                    let explode = if suffix.ends_with('!') {
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
                        score += i64::from(amount);
                    } else if min < max {
                        for _ in 0..amount.abs() {
                            score += signum * self.roll(min, max, explode, cmp, threshold);
                        }
                    }
                }
            }
        }
        Ok(score)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rolls_properly() {
        let mut roll = Roll::new();
        let mut rng = rand::thread_rng();
        for _ in 0..crate::FUZZ {
            let x = rng.gen();
            let y = rng.gen();
            let (min, max) = if x <= y { (x, y) } else { (y, x) };

            let result = roll.roll(min, max, false, 0, 0);

            assert!(result >= min);
            assert!(result <= max);
        }
    }
}
