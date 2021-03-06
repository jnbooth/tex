use chrono::{DateTime, Utc};
use diesel::dsl::sum;
use getopts::{Matches, Options};

use super::*;
use crate::db::{Conn, Page, page, pages};
use crate::util;

pub struct Search {
    opts: Options
}

impl Command for Search {
    fn cmds(&self) -> Vec<String> {
        own(&["search", "searc", "sear", "sea", "s"]) // but not se(en)
    }
    fn usage(&self) -> String { "<query> [-a <author>] [-t <tag>] [-t <another>] [-< <before MM-DD-YYYY>] [-> <after MM-DD-YYYY>] [-e <exclude>] [-e <another>] [-u]".to_owned() }
    fn fits(&self, size: usize) -> bool { size >= 1 }
    fn auth(&self) -> Auth { Anyone }

    fn run(&mut self, args: &[&str], _: &Context, db: &mut Db) -> Outcome {
        let opts = self.opts.parse(args)?;
        let conn = db.conn()?;

        let size = pages::filter(&opts, page::table.count())?.get_result(&conn)?;

        match size {
            0 => Err(NoResults),
            1 => Ok(vec![Reply(self.show_result(&opts, &conn, db)?)]),
            _ if opts.opt_present("u") => {
                let authors = pages::filter(&opts, page::table
                    .select(page::created_by)
                    .distinct_on(page::created_by)
                )?.execute(&conn)?;
                let votes: Option<i64> = pages::filter(&opts, page::table
                    .select(sum(page::rating))
                )?.get_result(&conn)?;
                let rating = votes.unwrap_or(0);
                let avg = rating / size;
                let earliest: DateTime<Utc> = pages::filter(&opts, page::table
                    .select(page::created_at)
                    .order(page::created_at.asc())
                )?.get_result(&conn)?;
                let latest: DateTime<Utc> = pages::filter(&opts, page::table
                    .select(page::created_at)
                    .order(page::created_at.desc())
                )?.get_result(&conn)?;
                let highest: Page = pages::filter(&opts, page::table
                    .order(page::rating.desc())
                )?.first(&conn)?;
                Ok(vec![Reply(format!(
                    "Found \x02{}\x02 pages by \x02{}\x02 authors. They have a total rating of \x02{}\x02, with an average of \x02{}\x02. The pages were created between {} ago and {} ago. The highest rated page is \x02{}\x02 at \x02{}\x02.",
                    size, authors, util::rating(rating), util::rating(avg), 
                    util::ago(earliest), util::ago(latest),
                    db.title(&highest), util::rating(i64::from(highest.rating))
                ))])
            },
            _ => Err(Ambiguous(size, 
                pages::filter(&opts, page::table
                    .select(page::title)
                    .order(page::created_at.desc())
                    .limit(20)
                )?.load(&conn)?
            ))
        }
    }
}

impl Search {
    #[inline]
    pub fn new() -> Self {
        Self { opts: pages::options() }
    }

    fn show_result(&self, opts: &Matches, conn: &Conn, db: &mut Db) -> Result<String, Error> {
        let page: Page = pages::filter(opts, page::table)?.first(conn)?;
        Ok(format!(
            "\x02{}\x02 (written {} ago by {}; \x02{}\x02) - http://{}/{}",
            db.title(&page),
            util::ago(page.created_at),
            page.created_by,
            util::rating(db.wiki.rate(&page.id, &db.client).ok_or(NoResults)?),
            db.wiki.root,
            page.id
        ))
    }
}
