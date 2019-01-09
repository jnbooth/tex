use lazy_static::lazy_static;
use multimap::MultiMap;
use regex::Regex;
use reqwest::Client;
use select::document::Document;
use select::predicate::{Class, Name};

use crate::{IO, util};

fn stringify(word: &str, defs: &MultiMap<String, String>) -> String {
    let mut s = String::new();
    s.push_str("\x02");
    s.push_str(word);
    s.push_str(":\x02");
    for (k, vs) in defs.iter_all() {
        s.push_str(" \x1d(");
        s.push_str(k);
        s.push_str(")\x1d");
        let mut i = 1;
        for v in vs {
            s.push_str(" ");
            s.push_str(&i.to_string());
            s.push_str(". ");
            s.push_str(v);
            i += 1;
        }
    }
    s
}

pub fn search(client: &Client, query: &str) -> IO<String> {
    let page = Document::from_read(
        client.get(&format!("http://ninjawords.com/{}", util::encode(query))).send()?
    )?;
    let mut defs = MultiMap::new();
    let mut article = "".to_owned();
    let word = page
        .find(Class("title-word"))
        .next()
        .ok_or(failure::err_msg("Word not found."))?
        .text()
        .trim()
        .to_owned();
    for node in page.find(Name("dd")) {
        if node.attr("class") == Some("article") {
            article = node.text()
        } else if let Some(entry) = node.find(Class("definition")).next() {
            let text = entry.text().trim()[2..].to_owned();
            if !text.starts_with('(') {
                defs.insert(article.to_owned(), clean_content(&text))
            }
        }
    }
    Ok(stringify(&word, &defs))
}

fn clean_content(s: &str) -> String {
    lazy_static! {
        static ref RE_SPACE: Regex = Regex::new(" \\([^()]+\\) ")
            .expect("RE_SPACE Regex failed to compile");
        static ref RE_ALL: Regex = Regex::new("\\s*\\([^()]+\\)\\s*")
            .expect("RE_ALL Regex failed to compile");
    };
    let mut clean = RE_SPACE.replace_all(s, " ").into_owned();
    clean = RE_ALL.replace_all(&clean, "").into_owned();
    if let Some(i) = clean.find(';') {
        clean.pop();
        clean = clean[0..i].to_owned();
        clean.push_str(".");
    }
    clean
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search() {
        assert_eq!(search(&Client::new(), "gulch").unwrap(), "\x02gulch:\x02 \x1d(noun)\x1d 1. A narrow V-shaped valley with a stream running through it. 2. A remote town or village, lacking in infrastructure and equipment.");
    }
    
    #[test]
    fn test_clean_content() {
        assert_eq!(
            clean_content("°The name for the fifth letter of the Greek alphabet, ε or Ε, preceded by delta (Δ, δ) and followed by zeta (Ζ, ζ). °In IPA, the phonetic symbol that represents the ; represented in SAMPA as E."),
            "°The name for the fifth letter of the Greek alphabet, ε or Ε, preceded by delta and followed by zeta. °In IPA, the phonetic symbol that represents the .".to_owned()
        );
    }

    #[test]
    fn test_not_found() {
        assert!(search(&Client::new(), "shisno").is_err());
    }
}
