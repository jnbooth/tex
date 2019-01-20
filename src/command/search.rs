use chrono::{DateTime, Utc};
use diesel::dsl::{exists, not};
use diesel::pg::Pg;
use diesel::query_builder::BoxedSelectStatement;
use getopts::{Matches, Options};

use super::*;
use crate::util;
use crate::wikidot::Wikidot;

pub struct Search {
    opts: Options,
    wiki: Wikidot
}

impl Command for Search {
    fn cmds(&self) -> Vec<String> {
        own(&["search", "searc", "sear", "sea", "s"]) // but not se(en)
    }
    fn usage(&self) -> String { "<query> [-a <author>] [-t <tag>] [-t <another>] [-< <before MM-DD-YYYY>] [-> <after MM-DD-YYYY>] [-e <exclude>] [-e <another>]".to_owned() }
    fn fits(&self, size: usize) -> bool { size >= 1 }
    fn auth(&self) -> i32 { 0 }

    fn run(&mut self, args: &[&str], _: &Context, db: &mut Db) -> Outcome {
        let opts = self.opts.parse(args)?;

        let size = db.get_result(Self::build_query(&opts, db::page::table.count().into_boxed())?)?;

        match size {
            0 => Err(NoResults),
            1 => Ok(vec![Reply(self.show_result(&opts, &db)?)]),
            _ if opts.opt_present("u") => {
                /*Found 9657 pages by 1626 authors. They have a total rating of +949397, with an average of +98. The pages were created between 10 years ago and 3 hours ago. The highest rated page is SCP-173: The Sculpture - The Original at +4319.
                */
                let authors = db.execute(Self::build_query(&opts, db::page::table
                    .select(db::page::created_by)
                    .distinct_on(db::page::created_by)
                    .into_boxed()
                )?)?;
                let earliest: DateTime<Utc> = db.get_result(Self::build_query(&opts, db::page::table
                    .select(db::page::created_at)
                    .order(db::page::created_at.asc())
                    .into_boxed()
                )?)?;
                let latest: DateTime<Utc> = db.get_result(Self::build_query(&opts, db::page::table
                    .select(db::page::created_at)
                    .order(db::page::created_at.desc())
                    .into_boxed()
                )?)?;
                Ok(vec![Reply(format!(
                    "Found {} pages by {} authors. The pages were created between {} ago and {} ago.", 
                    size, authors, util::ago(earliest), util::ago(latest)
                ))])
            },
            _ => Err(Ambiguous(size, 
                db.load(Self::build_query(&opts, db::page::table
                    .select(db::page::title)
                    .order(db::page::created_at.desc())
                    .limit(20)
                    .into_boxed()
                )?)?
            ))
        }
    }
}

impl Search {
    pub fn new(wiki: Wikidot) -> Self {
        let mut opts = Options::new();
        opts.optmulti("e", "exclude", "Exclude page titles", "TITLES");
        opts.optmulti("t", "tags", "Limit to certain tags", "TAGS");
        opts.optopt("a", "author", "Limit to an author", "AUTHOR");
        opts.optopt("<", "before", "Limit to pages published before a certain date.", "MM-DD-YYYY");
        opts.optopt(">", "after", "Limit to pages published after a certain date.", "YYYY-MM-DD");
        opts.optflag("u", "summary", "Summarize results.");
        //opts.optopt("r", "rating", "Limit to a range of ratings", "SCORE"); // TODO
        // opts.optopt("s", "strict", "Match exact words", "WORDS");
        //opts.optopt("f", "fullname", "Match an exact full name", "TITLE")
        Self { wiki, opts }
    }

    fn build_query<'a, T>(opts: &Matches, q: BoxedSelectStatement<'a, T, db::page::table, Pg>) 
    -> Result<BoxedSelectStatement<'a, T, db::page::table, Pg>, Error> {
        let mut query = q;
        /*
        let mut query = db::page::table
            //.distinct_on((db::page::fullname, db::page::created_at))
            .into_boxed();*/
        
        for free in &opts.free {
            query = query.filter(db::page::title.ilike(format!("%{}%", free)));
        }

        for tag in opts.opt_strs("t") {
            query = query.filter(exists(
                db::tag::table
                    .filter(db::tag::page.eq(db::page::fullname))
                    .filter(db::tag::name.eq(tag))
            ));
        }

        for tag in opts.opt_strs("e") {
            query = query.filter(not(db::page::title.ilike(format!("%{}%", tag))));
        }

        if let Some(author) = opts.opt_str("a") {
            query = query.filter(db::page::created_by.ilike(format!("%{}%", author)));
        }

        if let Some(before) = opts.opt_str("<") {
            let date = util::parse_date(&before).ok_or(InvalidArgs)?;
            query = query.filter(db::page::created_at.lt(date));
        }

        if let Some(after) = opts.opt_str(">") {
            let date = util::parse_date(&after).ok_or(InvalidArgs)?;
            query = query.filter(db::page::created_at.gt(date));
        }
        Ok(query)
    }

    fn show_result(&self, opts: &Matches, db: &Db) -> Result<String, Error> {
        let page: db::Page = db.first(Self::build_query(opts, db::page::table.into_boxed())?)?;
        Ok(format!(
            "{} (written {} ago by {}; {}) - http://{}/{}",
            page.title,
            util::ago(page.created_at),
            page.created_by,
            util::rating(self.wiki.rate(&page.fullname, &db.client).ok_or(NoResults)?),
            self.wiki.root,
            page.fullname
        ))
    }
}
