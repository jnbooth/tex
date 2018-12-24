use simple_error::SimpleError;
use std::env;
use xmlrpc::Request;
use xmlrpc::Value;
use super::IO;

pub struct Wikidot {
    url: String
}

impl Wikidot {
    pub fn new() -> Option<Wikidot> {
        let user = env::var("WIKIDOT_USER").ok()?;
        let key = env::var("WIKIDOT_KEY").ok()?;
        Some(Wikidot { url: format!("https://{}:{}@www.wikidot.com/xml-rpc-api.php", user, key) })
    }
    
    fn xml_rpc(&self, method: &str, params: Vec<(&str, Value)>) -> Result<Value, xmlrpc::Error> {
    Request::new(method)
        .arg(Value::Struct(params.into_iter().map(|(k, v)| (k.to_owned(), v)).collect()))
        .call_url(&self.url)
    }

    pub fn get_votes(&self, article: &str) -> IO<i32> {
        let res = self.xml_rpc("pages.get_meta", vec![
            ("site",  Value::from("scp-wiki.wikidot.com")),
            ("pages", Value::Array(vec![Value::from(article)]))
        ])?;
        let rating = parse_votes(&res, article)
            .ok_or(SimpleError::new(format!("Unable to parse {}", article)))?;
        Ok(rating)
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
