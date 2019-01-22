use diesel::dsl::exists;
use getopts::{Matches, Options};

use super::*;
use crate::db::pages;
use crate::util;
use crate::wikidot::Wikidot;

pub struct Author {
    opts: Options,
    wiki: Wikidot
}

impl Command for Author {
    fn cmds(&self) -> Vec<String> {
        abbrev("author")
    }
    fn usage(&self) -> String { "[<author>] [-t <tag>] [-t <another>] [-< <before MM-DD-YYYY>] [-> <after MM-DD-YYYY>] [-e <exclude>] [-e <another>]".to_owned() }
    fn fits(&self, _: usize) -> bool { true }
    fn auth(&self) -> i32 { 0 }

    fn run(&mut self, args: &[&str], ctx: &Context, db: &mut Db) -> Outcome {
        let mut opts = self.opts.parse(args)?;
        let free = opts.free.clone();
        opts.free.clear();
        let result = match free.as_slice() {
            []           => self.tally(&ctx.nick, &opts, db),
            [author_pat] => self.tally(author_pat, &opts, db),
            _            => Err(InvalidArgs)
        }?;
        Ok(vec![Reply(result)])
    }
}

impl Author {
    pub fn new(wiki: Wikidot) -> Self {
        Self { wiki, opts: pages::options() }
    }
    
    fn tally(&self, author_pat: &str, opts: &Matches, db: &Db) -> Result<String, Error> {
        let authors = Self::authors(author_pat, db)?;
        let author = match authors.as_slice() {
            [author] => Ok(author),
            _        => Err(NoResults)
        }?;
        let scps = Self::tagged("scp", author, opts, db)?;
        let tales = Self::tagged("tale", author, opts, db)?;
        let gois = Self::tagged("goi-format", author, opts, db)?;
        let hubs = Self::tagged("hub", author, opts, db)?;
        let art = Self::tagged("artwork", author, opts, db)?;
        let scps_len = scps.len();
        let tales_len = tales.len();
        let gois_len = gois.len();
        let hubs_len = hubs.len();
        let art_len = art.len();

        let mut all: Vec<db::Page> = [scps, tales, gois, hubs, art].concat();
        all.sort();
        all.dedup();

        let all_len = all.len();

        let mut votes = 0;
        let mut latest = all.first().ok_or(NoResults)?.clone();

        for page in all {
            votes += i64::from(page.rating);
            if page.created_at < latest.created_at {
                latest = page;
            }
        }

        let recent = self.wiki.rate(&latest.fullname, &db.client).ok_or(NoResults)?;

        let mut s = "\x02".to_owned();
        s.push_str(author);
        s.push_str("\x02 has \x02");
        s.push_str(&all_len.to_string());
        s.push_str("\x02 pages (");
        
        let mut comma = count(false, &mut s, scps_len, "SCP article");
        comma = count(comma, &mut s, tales_len, "tale");
        comma = count(comma, &mut s, gois_len, "GOI article");
        comma = count(comma, &mut s, hubs_len, "hub");
        count(comma, &mut s, art_len, "artwork page");

        s.push_str("). They have \x02");
        s.push_str(&votes.to_string());
        s.push_str("\x02 net votes with an average of \x02");
        s.push_str(&util::rating(votes / all_len as i64));
        s.push_str("\x02. Their latest page is \x02");
        s.push_str(&latest.title);
        s.push_str("\x02 at \x02");
        s.push_str(&util::rating(recent));
        s.push_str("\x02.");

        Ok(s)
    }

    fn authors(author_pat: &str, db: &Db) -> QueryResult<Vec<String>> {
        db.load(db::page::table
            .select(db::page::created_by)
            .filter(db::page::created_by.ilike(format!("%{}%", author_pat)))
            .distinct_on(db::page::created_by)
        )
    }

    fn tagged(tag: &str, author: &str, opts: &Matches, db: &Db) -> Result<Vec<db::Page>, Error> {
        Ok(db.load(pages::filter(opts, db::page::table
                .filter(db::page::created_by.eq(author))
                .filter(exists(
                    db::tag::table
                        .filter(db::tag::page.eq(db::page::fullname))
                        .filter(db::tag::name.eq(tag))
                ))
            )?)?
        )
    }
}

fn count(comma: bool, s: &mut String, size: usize, name: &str) -> bool {
    if size == 0 {
        comma
    } else {
        if comma {
            s.push_str(", ");
        }
        s.push_str("\x02");
        s.push_str(&size.to_string());
        s.push_str("\x02 ");
        s.push_str(name);
        if size != 1 {
            s.push_str("s");
        }
        true
    }
}
