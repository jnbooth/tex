use regex::Regex;
use reqwest::Client;
use serde_json::{Map, Value};
use lazy_static::lazy_static;

use crate::db::Db;
use crate::{IO, util};
use super::choice;

fn encode(s: &str) -> String {
    util::encode(&s.replace(" ", "_"))
}

fn clean_content(s: &str) -> String {
    lazy_static! {
        static ref RE_WIKI: Regex = Regex::new("\\s*\\([^()]+\\)")
            .expect("RE_WIKI Regex failed to compile");
    }
    RE_WIKI.replace_all(&s.replace("(listen)", ""), "").replace("  ", " ")
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
    let mut verbatim = links.to_owned()
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
    let extract = result.get("extract")?.as_str()?;
    
    let top = extract.split('\n').next()?;
    if top.ends_with(":") && top.contains("refer") {
        if let Some(disambig) = get_disambig(title, result) {
            return Some(Err(disambig))
        }
    }
    Some( Ok(
        util::trim(&format!(
            "{} \x02{}\x02: {}", 
            format!("https://en.wikipedia.org/wiki/{}", encode(title)), 
            title, 
            clean_content(&extract.replace("\n", " "))
        ))
    ) )
}

fn search_in(query: &str) -> IO<Result<String, Vec<String>>> {
    let client = Client::new();
    let searches = serde_json::from_reader(
        client.get(&format!(
            "https://en.wikipedia.org/w/api.php?format=json&formatversion=2&action=query&list=search&srlimit=1&srprop=&srsearch={}",
            encode(query)
        )).send()?
    )?;
    let page = get_page(&searches).ok_or(failure::err_msg("Page not found"))?;
    let entry = serde_json::from_reader(
        client.get(&format!(
            "https://en.wikipedia.org/w/api.php?format=json&action=query&prop=extracts|links&pllimit=100&exintro&explaintext&redirects=1&pageids={}",
            encode(&page.to_string())
        )).send()?
    )?;
    Ok(get_entry(page, &entry).ok_or(failure::err_msg("Entry not found"))?)
}

pub fn search(db: &mut Db, query: &str) -> IO<String> {
    match search_in(query)? {
        Ok(entry)  => Ok(entry),
        Err(ambig) => {
            db.choices.clear();
            let suggests = choice::suggest(&ambig);
            for link in ambig {
                db.choices.add(move || match search_in(&link) {
                    Ok(Ok(entry)) => Ok(entry),
                    Ok(Err(_))    => Err(failure::err_msg("Couldn't disambiguate.")),
                    Err(e)        => Err(e)
                })
            }
            Ok(suggests)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page() {
        assert_eq!(search(&mut Db::new(), "Monty Oum").unwrap(), "https://en.wikipedia.org/wiki/Monty_Oum \x02Monty Oum\x02: Monyreak \"Monty\" Oum was an American web-based animator and writer. A self-taught animator, he scripted and produced several crossover fighting video series, drawing the attention of internet production company Rooster Teeth, who hired him. […]");
    }

    #[test]
    fn test_ambig() {
        assert!(search(&mut Db::new(), "rock").unwrap().starts_with("Did you mean: (1)"));
    }
}
