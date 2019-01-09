use chrono::NaiveDate;
use chrono::offset::Local;
use multimap::MultiMap;
use reqwest::Client;
use select::document::Document;
use select::node::Node;
use select::predicate::{Class, Name};
use std::borrow::ToOwned;

use crate::{IO, env};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Ban {
    nicks:  Vec<String>,
    hosts:  Vec<String>,
    status: Option<NaiveDate>,
    reason: String
}
impl Ban {
    pub fn new(node: &Node) -> Option<Self> {
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
        Some(Ban { nicks, hosts, status, reason })
    }
    pub fn active(&self) -> bool {
        match self.status {
            None         => true,
            Some(expiry) => expiry >= Local::today().naive_local()
        }
    }
    pub fn matches(&self, nick: &String, host: &String) -> bool {
        self.nicks.contains(nick) || self.hosts.contains(host)
    }
}

pub struct Bans(MultiMap<String, Ban>);

impl Bans {
    pub fn new() -> Option<Bans> {
        Some(Bans(load_bans(&env::opt("BAN_PAGE")?).ok()?))
    }
    pub fn get_ban(&self, channel_up: &str, nick_up: &str, host_up: &str) -> Option<String> {
        let bans = self.0.get_vec(&channel_up.to_lowercase())?;
        let nick = nick_up.to_lowercase();
        let host = host_up.to_lowercase();
        let ban = bans.into_iter().filter(|x| x.active() && x.matches(&nick, &host)).next()?;
        Some(ban.reason.to_owned())
    }
}

fn load_bans(page: &str) -> IO<MultiMap<String, Ban>> {
    let mut bans = MultiMap::new();
    let doc = Document::from_read(Client::new().get(page).send()?)?;
    for node in doc.find(Class("wiki-content-table")) {
        if let Some(title) = node.find(Name("th")).next() {
            let chantext = title.text().to_owned();
            let chans = chantext.split(' ');
            for tr in node.find(Name("tr")) {
                if let Some(ban) = Ban::new(&tr) {
                    if ban.active() {
                        for chan in chans.to_owned() {
                            bans.insert(chan.to_owned(), ban.to_owned())
                        }
                    }
                }
            }
        }
    }
    Ok(bans)
}

