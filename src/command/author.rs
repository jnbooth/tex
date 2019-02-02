use getopts::{Matches, Options};
use std::borrow::ToOwned;

use super::*;
use crate::db::{Conn, Page, attribution, page, pages, tag};
use crate::util;

pub struct Author {
    opts: Options
}

impl Command for Author {
    fn cmds(&self) -> Vec<String> {
        abbrev("author")
    }
    fn usage(&self) -> String { "[<author>] [-t <tag>] [-t <another>] [-< <before MM-DD-YYYY>] [-> <after MM-DD-YYYY>] [-e <exclude>] [-e <another>]".to_owned() }
    fn fits(&self, _: usize) -> bool { true }
    fn auth(&self) -> u8 { 0 }

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
    pub fn new() -> Self {
        Self { opts: pages::options() }
    }
    
    fn tally(&self, author_pat: &str, opts: &Matches, db: &mut Db) -> Result<String, Error> {
        let conn = db.conn();
        let mut authors = page::table
            .filter(page::created_by.ilike(author_pat))
            .select(page::created_by)
            .load(&conn)?;
        authors.append(&mut attribution::table
            .filter(attribution::user.ilike(author_pat))
            .select(attribution::user)
            .load(&conn)?
        );
        authors.dedup();
        let author = match authors.as_slice() {
            []       => Err(NoResults),
            [author] => Ok(author),
            _        => Err(Ambiguous(authors.len() as i64, authors))
        }?;
        let scps = Self::tagged("scp", author, opts, &conn)?;
        let tales = Self::tagged("tale", author, opts, &conn)?;
        let gois = Self::tagged("goi-format", author, opts, &conn)?;
        let hubs = Self::tagged("hub", author, opts, &conn)?;
        let art = Self::tagged("artwork", author, opts, &conn)?;
        let scps_len = scps.len();
        let tales_len = tales.len();
        let gois_len = gois.len();
        let hubs_len = hubs.len();
        let art_len = art.len();

        let mut all: Vec<Page> = [scps, tales, gois, hubs, art].concat();
        all.sort();
        all.dedup();

        let all_len = all.len();

        let mut votes = 0;
        let mut latest = all.first().ok_or(NoResults)?.clone();

        for page in all {
            votes += i64::from(page.rating);
            if page.created_at > latest.created_at {
                latest = page;
            }
        }

        let recent = db.wiki.rate(&latest.id, &db.client).ok_or(NoResults)?;

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
        s.push_str(&db.title(&latest));
        s.push_str("\x02 at \x02");
        s.push_str(&util::rating(recent));
        s.push_str("\x02.");

        Ok(s)
    }

    fn tagged(tag: &str, author: &str, opts: &Matches, conn: &Conn) 
    -> Result<Vec<Page>, Error> {
        Ok(pages::filter_by(author, pages::filter(opts, page::table
                .filter(page::id.eq_any(
                    tag::table
                        .select(tag::page_id)
                        .filter(tag::name.eq(tag))
                ))
            )?).load(conn)?
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
