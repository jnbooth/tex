use diesel::pg::PgConnection;
use diesel::prelude::*;
use reqwest::Client;
use select::document::Document;
use select::node::Node;
use select::predicate::{Class, Name, Predicate};

use crate::{IO, db};

pub fn update(page: &str, cli: &Client, conn: &PgConnection) -> IO<()> {
    let doc = Document::from_read(cli.get(page).send()?)?;
    let attrs: Vec<db::Attribution> = doc
        .find(Class("wiki-content-table").descendant(Name("tr")))
        .filter_map(parse) 
        .collect();
    diesel::insert_into(db::attribution::table)
        .values(&attrs)
        .on_conflict_do_nothing()
        .execute(conn)?;
    Ok(())
}

fn parse(tr: Node) -> Option<db::Attribution> {
    let mut tds = tr.find(Name("td"));
    let page = tds.next()?.text();
    let user = tds.next()?.text().to_lowercase();
    let kind = tds.next()?.text();
    Some(db::Attribution { page, user, kind })
}
