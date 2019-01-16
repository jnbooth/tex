use hashbrown::HashSet;
use reqwest::Client;
use reqwest::header::AUTHORIZATION;
use select::document::Document;
use xmlrpc::Value;

use crate::{IO, env};
use crate::db::Page;

pub mod titles;
pub mod pages;

#[derive(Debug, Clone)]
pub struct Wikidot {
    pub root: String,
    pub site: String,
    ajax:     String,
    rpc:      String,
    auth:     String
}

impl Wikidot {
    pub fn new() -> Option<Self> {
        let root = env::opt("WIKIDOT_ROOT")?;
        let site = root.split('.').next()?;
        let api = env::api("WIKIDOT", "USER", "KEY")?;
        let auth = format!("Basic {}", base64::encode(&format!("{}:{}", api.user, api.key)));
        Some(Wikidot {
            ajax:   format!("http://{}.wikidot.com/ajax-module-connector.php", site),
            root:   root.to_owned(),
            rpc:    format!("https://www.wikidot.com/xml-rpc-api.php"),
            site:   site.to_owned(),
            auth
        })
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
        let body = get_body(&json).ok_or(failure::err_msg(
            format!("Invalid response from {} for {:?}", module_name, args))
        )?;
        Ok(Document::from(body))
    }

    pub fn get(&self, articles: &Vec<String>, cli: &Client) -> Result<Vec<Page>, xmlrpc::Error> {
        let pages = articles.into_iter().map(|x| Value::from(x.to_owned())).collect();
        let res = self.xml_rpc(cli, "pages.get_meta", vec![
            ("site",  Value::from(self.site.to_owned())),
            ("pages", Value::Array(pages))
        ])?;
        Ok(res
            .as_struct()
            .expect("Invalid pages.get_meta response")
            .into_iter()
            .filter_map(|(_, v)| Page::new(v))
            .collect()
        )
    }

    pub fn list(&self, cli: &Client) -> Result<HashSet<String>, xmlrpc::Error> {
        let res = self.xml_rpc(&cli, "pages.select", vec![
            ("site",  Value::from(self.site.to_owned())),
            ("parent", Value::from("-".to_owned())),
            ("order", Value::from("created_at desc".to_owned()))
        ])?;
        Ok(res
            .as_array()
            .expect("Invalid pages.select response")
            .into_iter()
            .filter_map(Value::as_str)
            .map(ToOwned::to_owned)
            .collect()
        )
    }

    pub fn rate(&self, title: &str, cli: &Client) -> Option<i32> {
        let res = self.xml_rpc(&cli, "pages.get_meta", vec![
            ("site",  Value::from(self.site.to_owned())),
            ("pages", Value::Array(vec![Value::from(title.to_owned())]))
        ]).ok()?;
        let (_, page) = res.as_struct()?.iter().next()?;
        Some(Page::new(page)?.rating)
    }

    pub fn walk<F>(&self, titles: &[String], cli: &Client, mut f: F) -> IO<()> 
    where F: FnMut(&str, Page, Vec<String>) -> IO<()> {
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
}

fn get_body(json: &serde_json::Value) -> Option<&str> {
    json.as_object()?.get("body")?.as_str()
}

