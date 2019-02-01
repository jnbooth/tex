use diesel::dsl::not;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::query_builder::BoxedSelectStatement;
use diesel::query_dsl::methods::BoxedDsl;
use getopts::{Options, Matches};

use crate::{db, util};
use crate::error::Error;
use crate::error::Error::*;

pub fn options() -> Options {
    let mut opts = Options::new();
    opts.optmulti("e", "exclude", "Exclude page titles", "TITLES");
    opts.optmulti("t", "tag", "Limit to certain tags", "TAGS");
    opts.optopt("a", "author", "Limit to an author", "AUTHOR");
    opts.optopt("<", "before", "Limit to pages published before a certain date.", "MM-DD-YYYY");
    opts.optopt(">", "after", "Limit to pages published after a certain date.", "YYYY-MM-DD");
    opts.optflag("u", "summary", "Summarize results.");
    //opts.optopt("r", "rating", "Limit to a range of ratings", "SCORE"); // TODO
    // opts.optopt("s", "strict", "Match exact words", "WORDS");
    //opts.optopt("f", "fullname", "Match an exact full name", "TITLE")
    opts
}


pub fn filter_by<'a, T>(author: &str, query: BoxedSelectStatement<'a, T, db::page::table, Pg>)
-> BoxedSelectStatement<'a, T, db::page::table, Pg> {
    query.filter(
        db::page::created_by.eq(author.to_owned())
        .or(db::page::id.eq_any(
            db::attribution::table
                .select(db::attribution::page_id)
                .filter(db::attribution::user.eq(author.to_owned()))
        ))
    )
}

pub fn filter<'a, B, T>(opts: &Matches, q: B)
-> Result<BoxedSelectStatement<'a, T, db::page::table, Pg>, Error> 
where B: QueryDsl + BoxedDsl<'a, Pg, Output = BoxedSelectStatement<'a, T, db::page::table, Pg>> {
    let mut query = q.into_boxed();
    
    for free in &opts.free {
        query = query.filter(db::page::title.ilike(format!("%{}%", free)));
    }

    for tag in opts.opt_strs("t") {
        query = query.filter(db::page::id.eq_any(
            db::tag::table
                .select(db::tag::page_id)
                .filter(db::tag::name.eq(tag))
        ));
    }

    for tag in opts.opt_strs("e") {
        query = query.filter(not(db::page::title.ilike(format!("%{}%", tag))));
    }

    
    if let Some(author) = opts.opt_str("a") {
        query = filter_by(&author.to_lowercase(), query);
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
