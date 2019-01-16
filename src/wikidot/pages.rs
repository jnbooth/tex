use hashbrown::HashSet;
use reqwest::Client;
use std::sync::mpsc::{Receiver, Sender, channel};

use crate::IO;
use crate::wikidot::Wikidot;

pub struct PagesDiff {
    client: Client,
    wiki:   Wikidot,
    sender: Sender<(String, bool)>,
    pages:  HashSet<String>
}

impl PagesDiff {
    pub fn build(wiki: Wikidot) -> IO<(Self, Receiver<(String, bool)>)> {
        let (sender, receiver) = channel();
        let client = Client::new();
        Ok((Self {
            sender, 
            pages: wiki.list(&client)?,
            wiki,
            client
        }, receiver))
    }

    pub fn diff(&mut self) -> IO<()> {
        let changed = self.wiki.list(&self.client)?;
        for added in changed.difference(&self.pages) {
            self.sender.send((added.to_owned(), true))?;
        }
        for deleted in self.pages.difference(&changed) {
            self.sender.send((deleted.to_owned(), false))?;
        }

        self.pages = changed;

        Ok(())
    }
    
    pub fn dup(&self) -> HashSet<String> {
        self.pages.clone()
    }
}
