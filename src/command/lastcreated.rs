use chrono::{DateTime, FixedOffset, NaiveDateTime};
use reqwest::Client;
use select::node::Node;
use select::predicate::{Class, Name, Predicate};
use std::time::SystemTime;

use super::*;
use crate::util;
use crate::wikidot::Wikidot;

const LIMIT: u8 = 3;
const TIMEZONE: i32 = 8 * 60 * 60;

pub struct LastCreated {
    wiki: Wikidot
}

impl<O: Output + 'static> Command<O> for LastCreated {
    fn cmds(&self) -> Vec<String> {
        own(&["lastcreated", "lc", "l"])
    }
    fn usage(&self) -> String { "".to_owned() }
    fn fits(&self, size: usize) -> bool { size == 0 }
    fn auth(&self) -> i32 { 0 }
    fn reload(&mut self, _: &mut Db) -> Outcome<()> { Ok(()) }

    fn run(&mut self, _: &[&str], irc: &O, ctx: &Context, db: &mut Db) -> Outcome<()> {
        for page in self.last_created(&db.client)? {
            irc.reply(ctx, &page)?;
        }
        Ok(())
    }
}

impl LastCreated {
    pub fn new(wiki: Wikidot) -> Self {
        LastCreated { wiki }
    }

    fn last_created(&self, client: &Client) -> Outcome<Vec<String>> {
        let pages = self.wiki.request_module("list/ListPagesModule", client, &[
            ("body", "title created_by created_at"),
            ("order", "created_at desc"),
            ("rating", ">=-10"),
            ("limit", &LIMIT.to_string())
        ]).map_err(Throw)?;
        Ok(pages.find(Class("list-pages-item")).filter_map(|x| self.parse_lc(&x)).collect())
    }    

    fn parse_lc(&self, val: &Node) -> Option<String> {
        let a = val.find(Name("h1").descendant(Name("a"))).next()?;
        let title = a.text();
        let link = a.attr("href")?;
        let author = val.find(Class("printuser").descendant(Name("a"))).last()?.text();
        let timestamp = val.find(Class("odate")).next()?.text();
        let ago = parse_time(&timestamp).ok()?;
        Some(format!("{} ({} ago by {}): http://{}{}", title, ago, author, self.wiki.root, link))
    }
}

fn parse_time(timestamp: &str) -> Outcome<String> {
    let naive = NaiveDateTime::parse_from_str(&timestamp, "%_d %b %Y %H:%M")?;
    let datetime: DateTime<FixedOffset> = DateTime::from_utc(naive, FixedOffset::west(TIMEZONE));
    Ok( util::since(SystemTime::from(datetime)) ?)
}
