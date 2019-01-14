#![allow(dead_code)]
use reqwest::Client;
use reqwest::header::AUTHORIZATION;
use select::document::Document;
use xmlrpc::Value;

use crate::env;

mod page;
pub mod titles;

use self::page::Page;

#[derive(Debug, Clone)]
pub struct Wikidot {
    ajax: String,
    rpc:  String,
    auth: String,
    pub root: String,
    pub site: String,
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

    pub fn get(&self, articles: &Vec<String>, cli: &Client) -> Result<Vec<Page>, failure::Error> {
        let pages = articles.into_iter().map(|x| Value::from(x.to_owned())).collect();
        let res = self.xml_rpc(cli, "pages.get_meta", vec![
            ("site",  Value::from(self.site.to_owned())),
            ("pages", Value::Array(pages))
        ])?;
        let obj = res.as_struct().ok_or_else(||failure::err_msg("Invalid response"))?;
        
        Ok(obj.into_iter().filter_map(|(_, v)| Page::new(v)).collect())
    }

    pub fn request_module(&self, module_name: &str, client: &Client, args: &[(&str, &str)]) 
    -> Result<Document, failure::Error> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::own;

    #[test] #[ignore]
    fn test_rating() {
        env::load();
        println!("{:?}",
            Wikidot::new().unwrap().get(&own(&["SCP-3191", "SCP-3209"]), &Client::new())
        )
    }
}
