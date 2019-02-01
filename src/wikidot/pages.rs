use hashbrown::HashSet;
use reqwest::Client;

use crate::{IO, db};
use crate::wikidot::Wikidot;
use crate::wikidot::diff::{Diff, DiffResult, DiffSender};

pub struct PagesDiff {
    client: Client,
    wiki:   Wikidot,
    sender: DiffSender<String>,
    pages:  HashSet<String>
}

impl Diff<String> for PagesDiff {
    fn new(sender: DiffSender<String>, _: &db::Pool) -> Self {
        Self {
            sender,
            client: Client::new(),
            pages:  HashSet::new(),
            wiki:   Wikidot::new(),   
        }
    }
    fn cache(&self) -> &HashSet<String> {
        &self.pages
    }
    fn refresh(&self) -> IO<HashSet<String>> {
        Ok(self.wiki.list(&self.client)?)
    }
    fn send(&self, k: String, v: bool) -> DiffResult<String> {
        self.sender.send((k, v))
    }
    fn update(&mut self, pages: HashSet<String>) {
        self.pages = pages;
    }
}
