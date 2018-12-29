use multimap::MultiMap;
use select::document::Document;
use select::predicate::{Class, Name};

use super::super::{IO, encode};

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
            i += 1;
        }
    }
    s
}

pub fn search(client: &reqwest::Client, query: &str) -> IO<String> {
    let search_res = client.get(&format!("http://ninjawords.com/{}", encode(query))).send()?;
    let page = Document::from_read(search_res)?;
    let mut defs = MultiMap::new();
    let mut article = "".to_owned();
    let title = page.find(Class("title-word")).next().ok_or(failure::err_msg("Word not found."))?;
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
