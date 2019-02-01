use hashbrown::HashSet;
use reqwest::Client;
use select::document::Document;
use select::node::Node;
use select::predicate::{Class, Name, Predicate, Text};

use crate::{IO, db};
use crate::wikidot::diff::{Diff, DiffResult, DiffSender};

pub struct TitlesDiff {
    client: Client,
    sender: DiffSender<(String, String)>,
    titles: HashSet<(String, String)>
}

impl Diff<(String, String)> for TitlesDiff {
    fn new(sender: DiffSender<(String, String)>, _: &db::Pool) -> Self {
        Self {
            sender,
            client: Client::new(),
            titles: HashSet::new()
        }
    }
    fn cache(&self) -> &HashSet<(String, String)> {
        &self.titles
    }
    fn refresh(&self) -> IO<HashSet<(String, String)>> {
        let mut pages: Vec<String> = 
            (2..6)
                .map(|i| format!("http://scp-wiki.wikidot.com/scp-series-{}", i))
                .collect();
        pages.push("http://scp-wiki.wikidot.com/scp-series".to_string());
        pages.push("http://www.scp-wiki.net/joke-scps".to_string());
        let mut titles = HashSet::new();
        for page in pages {
            let doc = Document::from_read(self.client.get(&page).send()?)?;
            for el in doc.find(Class("series").descendant(Name("li"))) {
                if let Some((k, v)) = parse_title(&el) {
                    if v != "[ACCESS DENIED]" {
                        titles.insert((k, v));
                    }
                }
            }
        }
        Ok(titles)
    }
    fn send(&self, k: (String, String), v: bool) -> DiffResult<(String, String)> {
        self.sender.send((k, v))
    }
    fn update(&mut self, titles: HashSet<(String, String)>) {
        self.titles = titles;
    }
}

fn parse_title(node: &Node) -> Option<(String, String)> {
    let link = node.find(Name("a")).next()?;
    let name = node.find(Name("span")).nth(1)
        .or_else(||node.find(Text).skip(1).find(|x| x.text().len() > 3))?;
    let name_text = name.text();
    let title = match name_text.find("- ") {
        None => name_text,
        Some(i) => name_text[i+2..].to_string()
    };
    Some((link.text().to_lowercase(), title))
}
