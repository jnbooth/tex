use percent_encoding::utf8_percent_encode;
use regex::Regex;
use serde_json::{Map, Value};
use simple_error::SimpleError;

use super::super::db::Db;
use super::super::IO;
use super::super::ErrIO;
use super::choice;

const CHARACTER_LIMIT: usize = 300;

fn encode(s: &str) -> String {
    utf8_percent_encode(&s.replace(" ", "_"), percent_encoding::DEFAULT_ENCODE_SET).to_string()
}

fn clean_content(s: &str) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new("\\s*\\([^()]+\\)").unwrap();
    }
    let mut content = RE.replace_all(&s.replace("(listen)", ""), "").replace("  ", " ");
    if content.len() > CHARACTER_LIMIT {
        if let Some(i) = content[..CHARACTER_LIMIT-4].rfind(' ') {
            content = content[..i].to_owned();
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

fn get_link(json: &Value) -> Option<String> {
    let link = json
        .as_object()?
        .get("title")?
        .as_str()?;
    if link.contains("disambiguation") {
        None
    } else {
        Some(link.to_owned())
    }
}

fn get_disambig(title_up: &str, json: &Map<String, Value>) -> Option<Vec<String>> {
    let title = format!("{} (", title_up.to_lowercase());
    let links = json
        .get("links")?
        .as_array()?
        .into_iter()
        .filter_map(get_link);
    let mut verbatim = links.clone()
        .filter(|x| x.to_lowercase().starts_with(&title))
        .peekable();
    if verbatim.peek().is_some() {
        Some(verbatim.collect())
    } else {
        Some(links.collect())
    }
}

fn get_entry(page: u64, json: &Value) -> Option<Result<String, Vec<String>>> {
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
    let top = extract.split("\n").next()?;
    
    if top.ends_with(":") && top.contains("refer") {
        if let Some(disambig) = get_disambig(title, result) {
            return Some(Err(disambig))
        }
    }
    Some( Ok(
        format!("\x02{}\x02 ({}) {}", title, link, clean_content(&extract.replace("\n", " ")))
    ) )
}

fn search_in(query: &str) -> IO<Result<String, Vec<String>>> {
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
pub fn search(db: &mut Db, query: &str) -> IO<String> {
    let searched = search_in(query)?;
    match searched {
        Ok(entry)  => Ok(entry),
        Err(ambig) => {
            db.choices.clear();
            let suggests = choice::suggest(&ambig);
            for link in ambig {
                db.choices.add(move || match search_in(&link) {
                    Ok(Ok(entry)) => Ok(entry),
                    Ok(Err(_))    => ErrIO("Couldn't disambiguate."),
                    Err(e)        => Err(e)
                })
            }
            Ok(suggests)
        }
    }
}
