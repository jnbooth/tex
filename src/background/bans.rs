use chrono::NaiveDate;
use chrono::offset::Local;
use hashbrown::HashSet;
use reqwest::Client;
use select::document::Document;
use select::node::Node;
use select::predicate::{Class, Name};
use std::iter::*;
use std::borrow::ToOwned;

use crate::{IO, env};
use super::diff::{Diff, DiffResult, DiffSender};

pub struct BansDiff {
    sender: DiffSender<(String, Ban)>,
    bans: HashSet<(String, Ban)>,
    page: String
}

impl Diff<(String, Ban)> for BansDiff {
    fn new(sender: DiffSender<(String, Ban)>) -> Self {
        Self { sender, bans: HashSet::new(), page: env::get("BAN_PAGE") }
    }
    fn cache(&self) -> &HashSet<(String, Ban)> {
        &self.bans
    }
    fn refresh(&self, cli: &Client) -> IO<HashSet<(String, Ban)>> {
        Ok(parse_bans(&Document::from_read(cli.get(&self.page).send()?)?))
    }
    fn send(&self, k: (String, Ban), v: bool) -> DiffResult<(String, Ban)> {
        self.sender.send((k, v))
    }
    fn update(&mut self, bans: HashSet<(String, Ban)>) {
        self.bans = bans;
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Ban {
    nicks:      Vec<String>,
    hosts:      Vec<String>,
    status:     Option<NaiveDate>,
    pub reason: String
}
impl Ban {
    pub fn build(node: &Node) -> Option<Self> {
        let mut tds = node.find(Name("td"));
        let nicks = tds
            .next()?
            .text()
            .split(' ')
            .filter(|x| !x.ends_with("-GENERIC"))
            .map(|x| x.to_lowercase())
            .collect();
        let hosts = tds
            .next()?
            .text()
            .split(' ')
            .filter_map(|x| x.to_lowercase().split('@').last().map(ToOwned::to_owned))
            .collect();
        let status = NaiveDate::parse_from_str(&tds.next()?.text(), "%m/%d/%Y").ok();
        let reason = tds.next()?.text();
        Some(Self { nicks, hosts, status, reason })
    }
    pub fn active(&self) -> bool {
        match self.status {
            None    => true,
            Some(t) => t >= Local::today().naive_local()
        }
    }
    #[allow(clippy::ptr_arg)] // Vec<String>, annoyingly, can only match against &String
    pub fn matches(&self, nick: &String, host: &String) -> bool {
        self.nicks.contains(nick) || self.hosts.contains(host)
    }
}

fn parse_bans(doc: &Document) -> HashSet<(String, Ban)> {
    let mut bans = HashSet::new();
    for node in doc.find(Class("wiki-content-table")) {
        if let Some(title) = node.find(Name("th")).next() {
            let chantext = title.text().to_owned();
            let chans = chantext.split(' ');
            for tr in node.find(Name("tr")) {
                if let Some(ban) = Ban::build(&tr) {
                    if ban.active() {
                        for chan in chans.clone() {
                            bans.insert((chan.to_owned(), ban.to_owned()));
                        }
                    }
                }
            }
        }
    }
    bans
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util;

    #[test]
    fn parses_bans() {
        env::load();
        assert!(!parse_bans(&util::webpage(&env::get("BAN_PAGE"))).is_empty());
    }
}
