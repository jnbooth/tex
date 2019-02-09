use reqwest::Client;
use reqwest::header::AUTHORIZATION;
use std::time::SystemTime;
use xmlrpc::Value;

use crate::{IO, env};
use crate::db::{Page, Tag};

#[derive(Debug, Clone)]
pub struct Wikidot {
    pub root: String,
    pub site: String,
    pub lc:   String,
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
            root:   root.to_owned(),
            rpc:    "https://www.wikidot.com/xml-rpc-api.php".to_owned(),
            site:   site.to_owned(),
            lc:     env::get("LC_PAGE"),
            auth
        }
    }

    fn xml_rpc(&self, cli: &Client, method: &str, params: Vec<(&str, Value)>) 
    -> Result<Value, xmlrpc::Error> {
        let req = cli.post(&self.rpc).header(AUTHORIZATION, self.auth.to_owned());
        xmlrpc::Request::new(method)
            .arg(Value::Struct(params.into_iter().map(|(k, v)| (k.to_owned(), v)).collect()))
            .call(req)
    }

    pub fn list(&self, cli: &Client) -> Result<Vec<String>, xmlrpc::Error> {
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
        Some(i64::from(Page::build(page, SystemTime::UNIX_EPOCH)?.rating))
    }

    pub fn walk<F>(&self, updated: SystemTime, titles: &[String], cli: &Client, mut f: F) -> IO<()> 
    where F: FnMut(Page, Vec<Tag>) -> IO<()> {
        for chunk in titles.chunks(10) {
            let pages = chunk.into_iter().map(|x| Value::from(x.to_owned())).collect();
            let res = self.xml_rpc(&cli, "pages.get_meta", vec![
                ("site",  Value::from(self.site.to_owned())),
                ("pages", Value::Array(pages))
            ])?;
            let obj = res.as_struct().ok_or_else(||failure::err_msg("Invalid response"))?;
            for v in obj.values() {
                if let Some((pg, tags)) = Page::tagged(v, updated) {
                    f(pg, tags)?;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_pages() {
        env::load();
        let wiki = Wikidot::new();
        let list = wiki.list(&Client::new()).expect("Error loading pages");
        println!("{}", list.len());
    }

    
    #[test] #[ignore]
    fn walks() {
        env::load();
        let cli = Client::new();
        let wiki = Wikidot::new();
        let list = wiki.list(&cli).expect("Error loading pages");
        let pages = list[..10].into_iter().map(|x| Value::from(x.to_owned())).collect();
        let res = wiki.xml_rpc(&cli, "pages.get_meta", vec![
            ("site",  Value::from(wiki.site.to_owned())),
            ("pages", Value::Array(pages))
        ]).expect("failed to respond");
        println!("{:?}", res);
    }
}
