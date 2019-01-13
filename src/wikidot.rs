#![allow(dead_code)]
use reqwest::Client;
use select::document::Document;
use xmlrpc::Value;

use crate::{IO, env};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Wikidot {
    ajax: String,
    rpc:  String,
    pub root: String,
    pub site: String
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

    pub fn request_module(&self, module_name: &str, client: &Client, args: &[(&str, &str)]) 
    -> IO<Document> {
        let mut full_args = args.to_owned();
        full_args.push(("moduleName", module_name));
        let res = client
            .post(&self.ajax)
            .form(&full_args)
            .send()?;
        let json: serde_json::Value = serde_json::from_reader(res)?;
        let body = get_body(&json).ok_or(failure::err_msg(
            format!("Invalid response from {} for {:?}", module_name, args))
        )?;
        Ok(Document::from(body))
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
