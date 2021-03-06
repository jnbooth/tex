use multimap::MultiMap;
use regex::Regex;
use select::document::Document;
use select::predicate::{Class, Name};

use super::*;
use crate::util;

pub struct Define {
    spaced: Regex,
    parens: Regex
}

impl Command for Define {
    fn cmds(&self) -> Vec<String> {
        abbrev("define")
    }
    fn usage(&self) -> String { "<query>".to_owned() }
    fn fits(&self, size: usize) -> bool { size >= 1 }
    fn auth(&self) -> Auth { Anyone }

    fn run(&mut self, args: &[&str], _: &Context, db: &mut Db) -> Outcome {
        Ok(vec![Reply(self.search(&args.join(" "), &db.client)?)])
    }
}

impl Default for Define { fn default() -> Self { Self::new() } }

impl Define {
    #[inline]
    pub fn new() -> Self {
        Self {
            spaced: Regex::new(" \\([^()]+\\) ").expect("Spaced regex failed to compile"),
            parens: Regex::new("\\s*\\([^()]+\\)\\s*").expect("Parens regex failed to compile")
        }
    }

    fn clean(&self, s: &str) -> String {
        let mut clean = self.spaced.replace_all(s, " ").into_owned();
        clean = self.parens.replace_all(&clean, "").into_owned();
        if let Some(i) = clean.find(';') {
            clean.pop();
            clean = clean[..i].to_owned();
            clean.push_str(".");
        }
        clean
    }

        
    fn search(&self, query: &str, cli: &reqwest::Client) -> Result<String, Error> {
        let page = Document::from_read(
            cli.get(&format!("http://ninjawords.com/{}", util::encode(query))).send()?
        )?;
        let word = page
            .find(Class("title-word"))
            .next()
            .ok_or_else(||ParseErr(err_msg("Missing title-word")))?;
        Ok(self.parse(word.text().trim(), &page))
    }
    fn parse(&self, word: &str, doc: &Document) -> String {
        let mut defs = MultiMap::new();
        let mut article = String::new();
        for node in doc.find(Name("dd")) {
            match node.attr("class") {
                Some("article") => article = node.text(),
                _ => if let Some(entry) = node.find(Class("definition")).next() {
                    let text = entry.text().trim()[2..].to_owned();
                    if !text.starts_with('(') {
                        defs.insert(article.to_owned(), self.clean(&text))
                    }
                }
            }
        }
        stringify(word, &defs)
    }
}

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
            s.push_str(" \x02");
            s.push_str(&i.to_string());
            s.push_str(".\x02 ");
            s.push_str(v);
            i += 1;
        }
    }
    s
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test] #[ignore]
    fn defines_word() {
        assert_eq!(Define::new().test_def("gulch").unwrap(), "\x02gulch:\x02 \x1d(noun)\x1d \x021.\x02 A narrow V-shaped valley with a stream running through it. \x022.\x02 A remote town or village, lacking in infrastructure and equipment.");
    }

    #[test] #[ignore]
    fn not_found() {
        assert!(Define::new().test_def("shisno").is_err());
    }
}
