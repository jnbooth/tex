use rand::Rng;
use simple_error::SimpleError;

use super::super::IO;
use super::super::ErrIO;

pub fn throw(s: &str) -> IO<String> {
    let mut throw = s.to_owned();
    if !throw.starts_with('+') && !throw.starts_with('-') {
        throw = format!("+ {}", throw);
    }
    let dice: Vec<&str> = throw.split(' ').filter(|x| !x.is_empty()).collect();
    if dice.len() & 1 != 0 {
        return ErrIO("Wrong number of arguments.");
    }
    let mut score: i64 = 0;
    let mut rng = rand::thread_rng();
    for chunk in dice.chunks_exact(2) {
        match chunk {
            &[sign, die] => {
                let signum: i64 = match sign {
                    "+" => Ok(1),
                    "-" => Ok(-1),
                    _   => Err(SimpleError::new(format!("{} is neither '+' nor '-'.", sign)))
                }?;
                match die.find('d') {
                    None    => {
                        let bonus: u32 = die.parse()?;
                        score = score + signum * bonus as i64;
                    },
                    Some(i) => {
                        let (amount_s, sides_s) = die.split_at(i);
                        let amount: u16 = if amount_s.is_empty() {
                            Ok(1)
                        } else {
                            amount_s.parse()
                        }?;
                        let sides: u16 = sides_s
                            .get(1..)
                            .ok_or(SimpleError::new("Number of sides not given."))?
                            .parse()?;
                        for _ in 0..amount {
                            score = score + signum * rng.gen_range(1, sides + 1) as i64;
                        }
                    }
                }
            },
            _ => return ErrIO("Chunking error.")
        }
    }
    Ok(score.to_string())
}
