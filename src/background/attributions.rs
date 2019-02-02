use diesel::query_dsl::RunQueryDsl;
use reqwest::Client;
use select::document::Document;
use select::node::Node;
use select::predicate::{Class, Name, Predicate};

use crate::IO;
use crate::db::{Attribution, Conn};
use crate::wikidot::Wikidot;
use crate::db::attribution;

const URL: &str = "http://www.scp-wiki.net/attribution-metadata";

fn parse(tr: Node) -> Option<Attribution> {
    let mut tds = tr.find(Name("td"));
    let page_id = tds.next()?.text();
    let user = tds.next()?.text().to_lowercase();
    let kind = tds.next()?.text();
    Some(Attribution { page_id, user, kind })
}

pub fn update(cli: &Client, conn: &Conn, _: &Wikidot) -> IO<()> {
    let doc = Document::from_read(cli.get(URL).send()?)?;
    let attrs: Vec<Attribution> = doc
        .find(Class("wiki-content-table").descendant(Name("tr")))
        .filter_map(parse)
        .filter(|x| x.kind != "maintainer")
        .collect();
    diesel::insert_into(attribution::table)
        .values(&attrs)
        .on_conflict_do_nothing()
        .execute(conn)?;
    Ok(())
}
