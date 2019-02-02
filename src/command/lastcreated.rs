use select::document::Document;
use select::predicate::{Class, Name, Predicate};

use std::time::SystemTime;

use super::*;
use crate::{IO, util};

const LIMIT: usize = 3;

pub struct LastCreated;

impl Command for LastCreated {
    fn cmds(&self) -> Vec<String> {
        own(&["lastcreated", "lc", "l"])
    }
    fn usage(&self) -> String { "".to_owned() }
    fn fits(&self, size: usize) -> bool { size == 0 }
    fn auth(&self) -> u8 { 0 }

    fn run(&mut self, _: &[&str], _: &Context, db: &mut Db) -> Outcome {
        last_created(&db)
    }
}

fn lc_titles(db: &Db) -> IO<Vec<String>> {
    let mut titles = Vec::new();

    let doc = Document::from_read(db.client.get(&db.wiki.lc).send()?)?;
    for node in doc.find(Class("list-pages-box").descendant(Name("td")).descendant(Name("a"))) {
        if let Some(href) = node.attr("href") {
            titles.push(href.to_owned());
        }
        if titles.len() >= LIMIT {
            break;
        }
    }

    Ok(titles)
}

fn last_created(db: &Db) -> Result<Vec<Response>, Error> {
    let mut responses = Vec::new();
    db.wiki.walk(SystemTime::now(), &lc_titles(db).map_err(Throw)?, &db.client, |page, _| {
        responses.push(Reply(format!(
            "\x02{}\x02 ({} ago by {}): http://{}/{}", 
            db.title(&page), util::ago(page.created_at), page.created_by, db.wiki.root, page.id
        )));
        Ok(())
    }).map_err(Throw)?;
    Ok(responses)
}
