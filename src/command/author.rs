use diesel::dsl::exists;
use super::*;
use crate::util;
use crate::wikidot::Wikidot;

pub struct Author {
    wiki: Wikidot
}

impl Command for Author {
    fn cmds(&self) -> Vec<String> {
        abbrev("author")
    }
    fn usage(&self) -> String { "<author>".to_owned() }
    fn fits(&self, size: usize) -> bool { size == 1 }
    fn auth(&self) -> i32 { 0 }

    fn run(&mut self, args: &[&str], _: &Context, db: &mut Db) -> Outcome {
        Ok(vec![Reply(self.tally(args[0], db)?)])
    }
}

impl Author {
    pub fn new(wiki: Wikidot) -> Self {
        Self { wiki }
    }
    
    fn tally(&self, author_pat: &str, db: &Db) -> Result<String, Error> {
        let authors: Vec<String> = db::page::table
            .select(db::page::created_by)
            .filter(db::page::created_by.ilike(format!("%{}%", author_pat)))
            .distinct_on(db::page::created_by)
            .load(&db.conn)?;
        let author = match authors.as_slice() {
            [author] => Ok(author),
            _        => Err(NoResults)
        }?;
        let latest: db::Page = db::page::table
            .filter(db::page::created_by.eq(author))
            .order_by(db::page::created_at.desc())
            .first(&db.conn)?;
        let scps = Self::tagged("scp", author, db)?;
        let tales = Self::tagged("tale", author, db)?;
        let hubs = Self::tagged("hub", author, db)?;
        let gois = Self::tagged("goi-format", author, db)?;
        let scps_len = scps.len();
        let tales_len = tales.len();
        let hubs_len = hubs.len();
        let gois_len = gois.len();

        let mut all: Vec<String> = [scps, tales, hubs, gois].concat();
        all.sort();
        all.dedup();

        let all_len = all.len();

        let votes = self.wiki.votes(&all, &db.client).ok_or(NoResults)?;

        let recent = self.wiki.rate(&latest.fullname, &db.client).ok_or(NoResults)?;

        let mut s = author.to_owned();
        let mut comma = false;
        s.push_str(" has ");
        s.push_str(&all_len.to_string());
        s.push_str(" pages (");
        if scps_len > 0 {
            s.push_str(&scps_len.to_string());
            s.push_str(" SCP articles");
            comma = true;
        }
        if tales_len > 0 {
            if comma { s.push_str(", ")};
            s.push_str(&tales_len.to_string());
            s.push_str(" tales");
            comma = true;
        }
        if gois_len > 0 {
            if comma { s.push_str(", ") };
            s.push_str(&gois_len.to_string());
            s.push_str(" GOI articles");
            comma = true;
        }
        if hubs_len > 0 {
            if comma { s.push_str(", ") };
            s.push_str(&hubs_len.to_string());
            s.push_str(" hubs");
        }
        s.push_str("). They have ");
        s.push_str(&votes.to_string());
        s.push_str(" net votes with an average of ");
        s.push_str(&util::rating(votes / all.len() as i32));
        s.push_str(". Their latest page is ");
        s.push_str(&latest.title);
        s.push_str(" at ");
        s.push_str(&util::rating(recent));
        s.push_str(".");

        Ok(s)
    }

    fn tagged(tag: &str, author: &str, db: &Db) -> QueryResult<Vec<String>> {
        Ok(db::page::table
            .filter(db::page::created_by.eq(author))
            .filter(exists(
                db::tag::table
                    .filter(db::tag::page.eq(db::page::fullname))
                    .filter(db::tag::name.eq(tag))
            ))
            .load(&db.conn)?
            .into_iter()
            .map(|x: db::Page| x.fullname)
            .collect())
    }
}

/*
djkaktus ( http://www.scp-wiki.net/djkaktus ) has 104 pages (101 Originals, 3 Rewrites) (70 SCP Articles, 31 Tales, 3 GOI-Format Articles). They have 24189 net upvotes with an average of +232. Their latest page is djkaktus's Proposal III at +311.

Jabyrwock: djkaktus - (http://www.scp-wiki.net/djkaktus) has 90 pages. (61 SCP articles, 29 Tales). They have 20054 net upvotes with an average of 222.  Their latest page is djkaktus's Proposal III at 311.
*/
