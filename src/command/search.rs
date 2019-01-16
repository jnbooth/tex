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

impl<O: Output + 'static> Command<O> for Search {
    fn cmds(&self) -> Vec<String> {
        own(&["search", "searc", "sear", "sea", "s"]) // but not se(en)
    }
    fn usage(&self) -> String { "<query> [-a <author>] [-t <tag>] [-t <another>] [-< <before MM-DD-YYYY>] [-> <after MM-DD-YYYY>] [-e <exclude>] [-e <another>]".to_owned() }
    fn fits(&self, size: usize) -> bool { size > 0 }
    fn auth(&self) -> i32 { 0 }
    fn reload(&mut self, _: &mut Db) -> Outcome<()> { Ok(()) }

    fn run(&mut self, args: &[&str], irc: &O, ctx: &Context, db: &mut Db) -> Outcome<()> {
        let opts = self.opts.parse(args)?;

        let size = Self::build_query(&opts)?
            .count()
            .get_result(&db.conn)?;

        match size {
            0 => Err(NoResults),
            1 => { irc.reply(&ctx, &self.show_result(&opts, &db)?)?; Ok(()) },
            _ => Err(Ambiguous(size, 
                Self::build_query(&opts)?
                    .select(db::page::title)
                    .order(db::page::created_at.desc())
                    .limit(20)
                    .load(&db.conn)?
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
        //opts.optopt("r", "rating", "Limit to a range of ratings", "SCORE"); // TODO
        // opts.optopt("s", "strict", "Match exact words", "WORDS");
        //opts.optopt("f", "fullname", "Match an exact full name", "TITLE")
        Self { wiki, opts }
    }

    fn build_query(opts: &Matches) 
    -> Outcome<BoxedSelectStatement<db::page::SqlType, db::page::table, Pg>> {
        let mut query = db::page::table
            //.distinct_on((db::page::fullname, db::page::created_at))
            .into_boxed();
        
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

    fn show_result(&self, opts: &Matches, db: &Db) -> Outcome<String> {
        let page: db::Page = Self::build_query(opts)?.first(&db.conn)?;
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
