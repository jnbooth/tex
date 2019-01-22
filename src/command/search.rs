use chrono::{DateTime, Utc};
use diesel::dsl::sum;
use getopts::{Matches, Options};

use super::*;
use crate::db::pages;
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
    fn usage(&self) -> String { "<query> [-a <author>] [-t <tag>] [-t <another>] [-< <before MM-DD-YYYY>] [-> <after MM-DD-YYYY>] [-e <exclude>] [-e <another>] [-u]".to_owned() }
    fn fits(&self, size: usize) -> bool { size >= 1 }
    fn auth(&self) -> i32 { 0 }

    fn run(&mut self, args: &[&str], _: &Context, db: &mut Db) -> Outcome {
        let opts = self.opts.parse(args)?;

        let size = db.get_result(pages::filter(&opts, db::page::table.count())?)?;

        match size {
            0 => Err(NoResults),
            1 => Ok(vec![Reply(self.show_result(&opts, &db)?)]),
            _ if opts.opt_present("u") => {
                let authors = db.execute(pages::filter(&opts, db::page::table
                    .select(db::page::created_by)
                    .distinct_on(db::page::created_by)
                )?)?;
                let votes: Option<i64> = db.get_result(pages::filter(&opts, db::page::table
                    .select(sum(db::page::rating))
                )?)?;
                let rating = votes.unwrap_or(0);
                let avg = rating / size;
                let earliest: DateTime<Utc> = db.get_result(pages::filter(&opts, db::page::table
                    .select(db::page::created_at)
                    .order(db::page::created_at.asc())
                )?)?;
                let latest: DateTime<Utc> = db.get_result(pages::filter(&opts, db::page::table
                    .select(db::page::created_at)
                    .order(db::page::created_at.desc())
                )?)?;
                let highest: db::Page = db.first(pages::filter(&opts, db::page::table
                    .order(db::page::rating.desc())
                )?)?;
                Ok(vec![Reply(format!(
                    "Found \x02{}\x02 pages by \x02{}\x02 authors. They have a total rating of \x02{}\x02, with an average of \x02{}\x02. The pages were created between {} ago and {} ago. The highest rated page is \x02{}\x02 at \x02{}\x02.",
                    size, authors, util::rating(rating), util::rating(avg), 
                    util::ago(earliest), util::ago(latest),
                    highest.title, util::rating(i64::from(highest.rating))
                ))])
            },
            _ => Err(Ambiguous(size, 
                db.load(pages::filter(&opts, db::page::table
                    .select(db::page::title)
                    .order(db::page::created_at.desc())
                    .limit(20)
                )?)?
            ))
        }
    }
}

impl Search {
    pub fn new(wiki: Wikidot) -> Self {
        Self { wiki, opts: pages::options() }
    }

    fn show_result(&self, opts: &Matches, db: &Db) -> Result<String, Error> {
        let page: db::Page = db.first(pages::filter(opts, db::page::table)?)?;
        Ok(format!(
            "\x02{}\x02 (written {} ago by {}; \x02{}\x02) - http://{}/{}",
            page.title,
            util::ago(page.created_at),
            page.created_by,
            util::rating(self.wiki.rate(&page.fullname, &db.client).ok_or(NoResults)?),
            self.wiki.root,
            page.fullname
        ))
    }
}
