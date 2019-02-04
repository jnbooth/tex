use hashbrown::HashSet;
use reqwest::Client;
use select::document::Document;
use select::node::Node;
use select::predicate::{Class, Name, Predicate, Text};
use std::iter::*;

use crate::IO;
use super::diff::{Diff, DiffResult, DiffSender};

pub struct TitlesDiff {
    sender: DiffSender<(String, String)>,
    titles: HashSet<(String, String)>
}

impl Diff<(String, String)> for TitlesDiff {
    fn new(sender: DiffSender<(String, String)>) -> Self {
        Self { sender, titles: HashSet::new() }
    }
    fn cache(&self) -> &HashSet<(String, String)> {
        &self.titles
    }
    fn refresh(&self, cli: &Client) -> IO<HashSet<(String, String)>> {
        let mut titles = HashSet::new();

        try_page("http://scp-wiki.wikidot.com/joke-scps", cli, &mut titles)?;
        try_page("http://scp-wiki.wikidot.com/scp-series", cli, &mut titles)?;

        for page in (2..).map(|i| format!("http://scp-wiki.wikidot.com/scp-series-{}", i)) {
            match try_page(&page, cli, &mut titles) {
                Ok(true) => (),
                _        => break
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

fn try_page(page: &str, cli: &Client, titles: &mut HashSet<(String, String)>) -> IO<bool> {
    Ok(parse_page(&Document::from_read(cli.get(page).send()?)?, titles))
}

fn parse_page(doc: &Document, titles: &mut HashSet<(String, String)>) -> bool {
    let mut changed = false;
    for el in doc.find(Class("series").descendant(Name("li"))) {
        if let Some((k, v)) = parse_title(&el) {
            if v != "[ACCESS DENIED]" {
                titles.insert((k, v));
                changed = true;
            }
        }
    }
    changed
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util;

    #[test]
    fn parses_titles() {
        let mut set = HashSet::new();
        parse_page(&util::webpage("http://scp-wiki.wikidot.com/scp-series"), &mut set);
        assert!(!set.is_empty());
    }
}
