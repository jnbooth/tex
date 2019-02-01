use reqwest::Client;
use reqwest::header::AUTHORIZATION;
use select::document::Document;
use std::iter::*;
use xmlrpc::Value;

use crate::{IO, env};
use crate::db::Page;

pub mod diff;
mod authors;
mod titles;
mod pages;

pub use self::diff::Diff;
pub use self::authors::AuthorsDiff;
pub use self::titles::TitlesDiff;
pub use self::pages::PagesDiff;

#[derive(Debug, Clone)]
pub struct Wikidot {
    pub root: String,
    pub site: String,
    ajax:     String,
    rpc:      String,
    auth:     String
}

impl Default for Wikidot { fn default() -> Self { Self::new() } }
impl Wikidot {
    pub fn new() -> Self {
        let root = env::get("WIKIDOT_ROOT");
        let site = root.split('.').next().expect("Invalid WIKIDOT_ROOT field in .env");
        let api = env::api("WIKIDOT", "USER", "KEY").expect("Missing Wikidot API fields in .env");
        let auth = format!("Basic {}", base64::encode(&format!("{}:{}", api.user, api.key)));
        Self {
            ajax:   format!("https://{}.wikidot.com/ajax-module-connector.php", site),
            root:   root.to_owned(),
            rpc:    "https://www.wikidot.com/xml-rpc-api.php".to_owned(),
            site:   site.to_owned(),
            auth
        }
    }

    fn xml_rpc(&self, cli: &Client, method: &str, params: Vec<(&str, Value)>) 
    -> Result<Value, xmlrpc::Error> {
        let req = cli.post(&self.rpc)
            .header(AUTHORIZATION, self.auth.to_owned());
        xmlrpc::Request::new(method)
            .arg(Value::Struct(params.into_iter().map(|(k, v)| (k.to_owned(), v)).collect()))
            .call(req)
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
        let body = get_body(&json).ok_or_else(||failure::err_msg(
            format!("Invalid response from {} for {:?}: {:?}", module_name, args, json))
        )?;
        Ok(Document::from(body))
    }

    pub fn list<T: FromIterator<String>>(&self, cli: &Client) -> Result<T, xmlrpc::Error> {
        let res = self.xml_rpc(&cli, "pages.select", vec![
            ("site",  Value::from(self.site.to_owned())),
            ("order", Value::from("created_at desc".to_owned()))
        ])?;
        Ok(res
            .as_array()
            .expect("Invalid pages.select response")
            .into_iter()
            .filter_map(|x| {
                let s = x.as_str()?;
                if s.starts_with("fragment:") {
                    None
                } else {
                    Some(s.to_owned())
                }
            })
            .collect()
        )
    }

    pub fn rate(&self, title: &str, cli: &Client) -> Option<i64> {
        let res = self.xml_rpc(&cli, "pages.get_meta", vec![
            ("site",  Value::from(self.site.to_owned())),
            ("pages", Value::Array(vec![Value::from(title.to_owned())]))
        ]).ok()?;
        let (_, page) = res.as_struct()?.iter().next()?;
        Some(i64::from(Page::build(page)?.rating))
    }

    #[cfg(not(test))]
    pub fn walk<T, F>(&self, titles: &[String], cli: &Client, mut f: F) -> IO<()> 
    where T: FromIterator<String>, F: FnMut(&str, Page, T) -> IO<()> {
        for chunk in titles.chunks(10) {
            let pages = chunk.into_iter().map(|x| Value::from(x.to_owned())).collect();
            let res = self.xml_rpc(&cli, "pages.get_meta", vec![
                ("site",  Value::from(self.site.to_owned())),
                ("pages", Value::Array(pages))
            ])?;
            let obj = res.as_struct().ok_or_else(||failure::err_msg("Invalid response"))?;
            for (k, v) in obj {
                if let Some((pg, tags)) = Page::tagged(v) {
                    f(k, pg, tags)?;
                }
            }
        }

        Ok(())
    }
    #[cfg(test)]
    pub fn walk<T, F>(&self, _: &[String], _: &Client, _: F) -> IO<()> 
    where T: FromIterator<String>, F: FnMut(&str, Page, T) -> IO<()> {
        Ok(())
    }
}

fn get_body(json: &serde_json::Value) -> Option<&str> {
    json.as_object()?.get("body")?.as_str()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test] #[ignore]
    fn lists_pages() {
        env::load();
        let wiki = Wikidot::new();
        let list: Vec<String> = wiki.list(&Client::new()).expect("Error loading pages");
        println!("{}", list.len());
    }
}
