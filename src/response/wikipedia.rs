use percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET};
use regex::Regex;
use serde_json::Value;
use simple_error::SimpleError;

use super::super::IO;

const CHARACTER_LIMIT: usize = 300;

fn encode(s: &str) -> String {
    utf8_percent_encode(&s.replace(" ", "_"), DEFAULT_ENCODE_SET).to_string()
}

fn clean_content(s: &str) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new("\\s*\\([^()]+\\)").unwrap();
    }
    let mut content = RE.replace_all(&s.replace("(listen)", ""), "").replace("  ", " ");
    if content.len() > CHARACTER_LIMIT {
        content = content[..CHARACTER_LIMIT-4].to_string();
        if let Some(i) = content.rfind(' ') {
            content = content[..i].to_string();
        }
        content.push_str(" […]");
    }
    content
}

fn get_page(json: &Value) -> Option<u64> {
    json
        .as_object()?
        .get("query")?
        .as_object()?
        .get("search")?
        .as_array()?
        .get(0)?
        .as_object()?
        .get("pageid")?
        .as_u64()
}

fn get_entry(page: u64, json: &Value) -> Option<String> {
    let result = json
        .as_object()?
        .get("query")?
        .as_object()?
        .get("pages")?
        .as_object()?
        .get(&page.to_string())?
        .as_object()?;
    let title = result.get("title")?.as_str()?;
    let link = format!("en.wikipedia.org/wiki/{}", encode(title));
    let extract = result.get("extract")?.as_str()?;
    Some(format!("\x02{}\x02 ({}) {}", title, link, clean_content(&extract.replace("\n", " "))))
}

pub fn search(query: &str) -> IO<String> {
    let client = reqwest::Client::new();
    let search_res = client.get(&format!(
        "https://en.wikipedia.org/w/api.php?format=json&formatversion=2&action=query&list=search&srlimit=1&srprop=&srsearch={}",
        encode(query)
    )).send()?;
    let search_json = serde_json::from_reader(search_res)?;
    let page = get_page(&search_json).ok_or(SimpleError::new("Page not found"))?;
    let entry_res = client.get(&format!(
        "https://en.wikipedia.org/w/api.php?format=json&action=query&prop=extracts|links&pllimit=100&exintro&explaintext&redirects=1&pageids={}",
        encode(&page.to_string())
    )).send()?;
    let entry_json = serde_json::from_reader(entry_res)?;
    Ok(get_entry(page, &entry_json).ok_or(SimpleError::new("Entry not found"))?)
}
