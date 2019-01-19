use regex::Regex;
use std::time::{Duration, SystemTime};

use super::*;

pub struct Remindme {
    offset: Regex
}

impl Command for Remindme {
    fn cmds(&self) -> Vec<String> {
        own(&["remindme", "remind", "r"])
    }
    fn usage(&self) -> String { "[<days>d][<hours>h][<minutes>m] message".to_owned() }
    fn fits(&self, size: usize) -> bool { size > 1 }
    fn auth(&self) -> i32 { 0 }

    fn run(&mut self, args: &[&str], ctx: &Context, db: &mut Db) -> Outcome {
        let offset = self.parse_offset(&args[0]).ok_or(InvalidArgs)?;
        let when = SystemTime::now() + offset;
        add_reminder(&args[1..].join(" "), when, ctx, db)?;
        Ok(vec![Action(format!("writes down {}'s reminder.", &ctx.nick))])
    }
}

impl Remindme {
    pub fn new() -> Self {
        Self { offset: Regex::new("\\d+").expect("Offset regex failed to compile") }
    }  

    pub fn parse_offset(&self, s: &str) -> Option<Duration> {
        let format: &str = &self.offset.replace_all(s, "*").into_owned();
        let mut groups = self.offset.find_iter(s);
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
}

fn yield_offset(d: u32, h: u32, m: u32) -> Option<Duration> {
    Some(Duration::from_secs(u64::from(60 * (m + 60 * (h + 24 * d)))))
}

fn next<'r, 't>(groups: &mut regex::Matches<'r, 't>) -> Option<u32> {
    groups.next()?.as_str().parse().ok()
}


fn add_reminder(message: &str, when: SystemTime, ctx: &Context, db: &mut Db) -> QueryResult<()> {
    let reminder = db::Reminder {
        user:    ctx.user.to_owned(),
        when,
        message: message.to_owned()
    };
    #[cfg(not(test))] diesel
        ::insert_into(db::reminder::table)
        .values(&reminder)
        .execute(&db.conn)?;
    db.reminders.insert(ctx.user.to_owned(), reminder);
    Ok(())
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_offset() {
        let remindme = Remindme::new();
        let zero = Some(Duration::from_secs(0));
        assert!(["0d0h0m", "0d0h", "0d0m", "0d", "0h0m", "0h", "0m"]
            .into_iter()
            .all(|x| remindme.parse_offset(x) == zero));
        assert_eq!(remindme.parse_offset("x0d0h0m"), None);
    }
}
