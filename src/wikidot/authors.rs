use diesel::prelude::*;
use hashbrown::HashSet;
use reqwest::Client;
use select::document::Document;
use select::node::Node;
use select::predicate::{Class, Name, Predicate};

use crate::{IO, db, env};
use crate::wikidot::diff::{Diff, DiffResult, DiffSender};

pub struct AuthorsDiff {
    pool:    db::Pool,
    client:  Client,
    sender:  DiffSender<String>,
    authors: HashSet<String>,
    url:     String
}

impl Diff<String> for AuthorsDiff {
    fn new(sender: DiffSender<String>, pool: &db::Pool) -> Self {
        Self {
            sender,
            pool:    pool.clone(),
            client:  Client::new(),
            authors: HashSet::new(),
            url:     env::get("ATTRIBUTION_PAGE")
        }
    }
    fn cache(&self) -> &HashSet<String> {
        &self.authors
    }
    fn refresh(&self) -> IO<HashSet<String>> {
        let conn = self.pool.get()?;
        let doc = Document::from_read(self.client.get(&self.url).send()?)?;
        let attrs: Vec<db::Attribution> = doc
            .find(Class("wiki-content-table").descendant(Name("tr")))
            .filter_map(parse)
            .filter(|x| x.kind != "maintainer")
            .collect();
        diesel::insert_into(db::attribution::table)
            .values(&attrs)
            .on_conflict_do_nothing()
            .execute(&conn)?;
        let mut authors: HashSet<String> = db::page::table
            .select(db::page::created_by)
            .distinct()
            .load(&conn)?
            .into_iter()
            .collect();
        for attr in attrs {
            authors.insert(attr.user);
        }
        Ok(authors)
    }
    fn send(&self, k: String, v: bool) -> DiffResult<String> {
        self.sender.send((k, v))
    }
    fn update(&mut self, authors: HashSet<String>) {
        self.authors = authors;
    }
}

fn parse(tr: Node) -> Option<db::Attribution> {
    let mut tds = tr.find(Name("td"));
    let page_id = tds.next()?.text();
    let user = tds.next()?.text().to_lowercase();
    let kind = tds.next()?.text();
    Some(db::Attribution { page_id, user, kind })
}
