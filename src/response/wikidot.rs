#![allow(dead_code)]
use chrono::{DateTime, FixedOffset, NaiveDateTime};
use reqwest::Client;
use select::document::Document;
use select::node::Node;
use select::predicate::{Class, Name, Predicate};
use std::time::SystemTime;
use xmlrpc::Value;

use crate::{IO, env, util};

const LAST_CREATED: u8 = 3;
const TIMEZONE: i32 = 8 * 60 * 60;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Wikidot {
    ajax: String,
    root: String,
    rpc:  String,
    site: String
}

impl Wikidot {
    pub fn new() -> Option<Self> {
        let root = env::opt("WIKIDOT_ROOT")?;
        let base = root.split('.').next()?;
        let api = env::api("WIKIDOT", "USER", "KEY")?;
        Some(Wikidot {
            ajax: format!("http://{}.wikidot.com/ajax-module-connector.php", base),
            root: root.to_owned(),
            rpc:  format!("https://{}:{}@www.wikidot.com/xml-rpc-api.php", api.user, api.key),
            site: format!("{}.wikidot.com", base)
        })
    }
    fn xml_rpc(&self, method: &str, params: Vec<(&str, Value)>) -> Result<Value, xmlrpc::Error> {
    xmlrpc::Request::new(method)
        .arg(Value::Struct(params.into_iter().map(|(k, v)| (k.to_owned(), v)).collect()))
        .call_url(&self.rpc)
    }

    pub fn get_votes(&self, article: &str) -> IO<i32> {
        let res = self.xml_rpc("pages.get_meta", vec![
            ("site",  Value::from(self.site.to_owned())),
            ("pages", Value::Array(vec![Value::from(article)]))
        ])?;
        let rating = parse_votes(&res, article)
            .ok_or(failure::err_msg(format!("Unable to parse {}", article)))?;
        Ok(rating)
    }

    fn request_module(&self, module_name: &str, client: &Client, args: &[(&str, &str)]) 
    -> IO<Document> {
        let mut full_args = args.to_owned();
        full_args.push(("moduleName", module_name));
        let res = client
            .post(&self.ajax)
            .form(&full_args)
            .send()?;
        let json: serde_json::Value = serde_json::from_reader(res)?;
        let body = get_body(&json).ok_or(failure::err_msg("Invalid response"))?;
        Ok(Document::from(body))
    }

    pub fn last_created(&self, client: &Client) -> IO<Vec<String>> {
        let pages = self.request_module("list/ListPagesModule", client, &[
            ("body", "title created_by created_at"),
            ("order", "created_at desc"),
            ("rating", ">=-10"),
            ("limit", &LAST_CREATED.to_string())
        ])?;
        Ok(pages.find(Class("list-pages-item")).filter_map(|x| self.parse_lc(&x)).collect())
    }    

    fn parse_lc(&self, val: &Node) -> Option<String> {
        let a = val.find(Name("h1").descendant(Name("a"))).next()?;
        let title = a.text();
        let link = a.attr("href")?;
        let author = val.find(Class("printuser").descendant(Name("a"))).last()?.text();
        let timestamp = val.find(Class("odate")).next()?.text();
        let ago = parse_time(&timestamp).ok()?;
        Some(format!("{} ({} ago by {}): http://{}{}", title, ago, author, self.root, link))
    }
}

fn parse_votes(val: &Value, article: &str) -> Option<i32> {
    val
        .as_struct()?
        .get(article)?
        .as_struct()?
        .get("rating")?
        .as_i32()
}

fn get_body(json: &serde_json::Value) -> Option<&str> {
    json.as_object()?.get("body")?.as_str()
}

fn parse_time(timestamp: &str) -> IO<String> {
    let naive = NaiveDateTime::parse_from_str(&timestamp, "%_d %b %Y %H:%M")?;
    let datetime: DateTime<FixedOffset> = DateTime::from_utc(naive, FixedOffset::west(TIMEZONE));
    Ok( util::since(SystemTime::from(datetime)) ?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new() -> Wikidot {
        env::load();
        Wikidot::new().expect("Error initializing Wikidot")
    }

    #[test]
    fn test_last_created() {
        println!("*** {}", new().last_created(&Client::new()).unwrap().join("\n*** "));
    }

    #[test]
    fn test_votes() {
        println!("{:?}", new().get_votes("SCP-3209"));
    }
}
