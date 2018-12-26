use multimap::MultiMap;
use percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET};
use select::document::Document;
use select::predicate::*;
use simple_error::SimpleError;

use super::super::IO;

fn encode(s: &str) -> String {
    utf8_percent_encode(s, DEFAULT_ENCODE_SET).to_string()
}

//rock  -  noun: 1. The naturally occurring aggregate of solid mineral matter that constitutes a significant part of the earth's crust. 2. A mass of stone projecting out of the ground or water. verb: 1. To move gently back and forth. 2. To cause to shake or sway violently. Synonyms: stone, cliff, boulder, pebble, more», foundation, support, gem, diamond, ice, ice cube, crack, afrikaner, rule, rule, distaff.
fn stringify(word: &str, defs: &MultiMap<String, String>) -> String {
    let mut s = String::new();
    s.push_str("\x02");
    s.push_str(word);
    s.push_str(":\x02");
    for (k, vs) in defs.iter_all() {
        s.push_str(" (");
        s.push_str(k);
        s.push_str(")");
        let mut i = 1;
        for v in vs {
            s.push_str(" ");
            s.push_str(&i.to_string());
            s.push_str(". ");
            s.push_str(v);
            i = i + 1;
        }
    }
    s
}

pub fn search(query: &str) -> IO<String> {
    let client = reqwest::Client::new();
    let search_res = client.get(&format!("http://ninjawords.com/{}", encode(query))).send()?;
    let page = Document::from_read(search_res)?;
    let mut defs = MultiMap::new();
    let mut article = "".to_owned();
    let title = page.find(Class("title-word")).next().ok_or(SimpleError::new("Word not found."))?;
    let word_line = title.text();
    let word = word_line.trim();
    for node in page.find(Name("dd")) {
        if node.attr("class") == Some("article") {
            article = node.text()
        } else if let Some(entry) = node.find(Class("definition")).next() {
            defs.insert(article.clone(), entry.text().trim()[2..].to_owned())
        }
    }
    Ok(stringify(word, &defs))
}
