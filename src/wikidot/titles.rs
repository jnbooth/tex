use hashbrown::HashMap;
use reqwest::Client;
use select::document::Document;
use select::node::Node;
use select::predicate::{Class, Name, Predicate, Text};
use std::sync::mpsc::{Receiver, Sender, channel};

use crate::IO;

pub struct TitlesDiff {
    client: Client,
    sender: Sender<(String, String)>,
    titles: HashMap<String, String>
}

impl TitlesDiff {
    pub fn build() -> IO<(Self, Receiver<(String, String)>)> {
        let (sender, receiver) = channel();
        let client = Client::new();
        Ok((Self {
            sender, 
            titles: record_titles(&client)?,
            client
        }, receiver))
    }

    pub fn dup(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        for (k, v) in &self.titles {
            if v != "ACCESS_DENIED" {
                map.insert(k.to_owned(), v.to_owned());
            }
        }
        map
    }

    pub fn diff(&mut self) -> IO<()> {
        let changed = record_titles(&self.client)?;
        for (k, v) in changed {
            if self.titles.insert(k.to_owned(), v.to_owned()).is_none() {
                self.sender.send((k, v))?;
            }
        }

        Ok(())
    }
}

fn parse_title(node: &Node) -> Option<(String, String)> {
    let link = node.find(Name("a")).next()?;
    let name = node.find(Name("span")).nth(1)
        .or_else(||node.find(Text).skip(1).find(|x| x.text().len() > 3))?;
    let name_text = name.text();
    match name_text.find("- ") {
            None => Some((link.text(), name_text)),
            Some(i) => Some((link.text(), name_text[i+2..].to_string()))
        }
}

/// Requests names for all articles from the mainlist.
fn record_titles(cli: &Client) -> IO<HashMap<String, String>> {
    let mut titles = HashMap::new();
    let mut pages: Vec<String> = 
        (2..6)
            .map(|i| format!("http://scp-wiki.wikidot.com/scp-series-{}", i))
            .collect();
    pages.push("http://scp-wiki.wikidot.com/scp-series".to_string());
    pages.push("http://www.scp-wiki.net/joke-scps".to_string());
    for page in pages {
        let res = cli.get(&page).send()?;
        let doc = Document::from_read(res)?;
        for node in doc.find(Class("series").descendant(Name("li"))) {
            if let Some((k, v)) = parse_title(&node) {
                titles.insert(k.to_lowercase(), v);
            }
        }
    }
    Ok(titles)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_titles() {
        let titles = record_titles(&Client::new());
        println!("{:?}", titles);
    }
}
