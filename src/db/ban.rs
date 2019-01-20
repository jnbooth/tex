use chrono::NaiveDate;
use chrono::offset::Local;
use hashbrown::HashSet;
use multimap::MultiMap;
use reqwest::Client;
use select::document::Document;
use select::node::Node;
use select::predicate::{Class, Name};
use std::borrow::ToOwned;

use crate::{Context, IO, env};

#[derive(Debug, Clone, PartialEq, Eq)]
struct Ban {
    nicks:  HashSet<String>,
    hosts:  HashSet<String>,
    status: Option<NaiveDate>,
    reason: String
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
    pub fn matches(&self, nick: &str, host: &str) -> bool {
        self.nicks.contains(nick) || self.hosts.contains(host)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bans(MultiMap<String, Ban>);

impl Bans {
    pub fn build() -> Option<Bans> {
        Some(Bans(load_bans(&env::opt("BAN_PAGE")?).ok()?))
    }
    pub fn get_ban(&self, ctx: &Context) -> Option<String> {
        let bans = self.0.get_vec(&ctx.channel)?;
        let ban = bans.into_iter()
            .find(|x| x.active() && x.matches(&ctx.user, &ctx.host))?;
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
                if let Some(ban) = Ban::build(&tr) {
                    if ban.active() {
                        for chan in chans.clone() {
                            bans.insert(chan.to_owned(), ban.to_owned())
                        }
                    }
                }
            }
        }
    }
    Ok(bans)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test] #[ignore]
    fn loads() {
        Bans::build().expect("Failed to load bans.");
    }
}
