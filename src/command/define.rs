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

impl<O: Output + 'static> Command<O> for Define {
    fn cmds(&self) -> Vec<String> {
        abbrev("define")
    }
    fn usage(&self) -> String { "<query>".to_owned() }
    fn fits(&self, size: usize) -> bool { size > 0 }
    fn auth(&self) -> i32 { 0 }
    fn reload(&mut self, _: &mut Db) -> Outcome<()> { Ok(()) }

    fn run(&mut self, args: &[&str], irc: &O, ctx: &Context, db: &mut Db) -> Outcome<()> {
        Ok(irc.reply(ctx, &self.search(&args.join(" "), &db.client)?)?)
    }
}

impl Define {
    pub fn new() -> Self {
        Define {
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

        
    fn search(&self, query: &str, cli: &reqwest::Client) -> Outcome<String> {
        let page = Document::from_read(
            cli.get(&format!("http://ninjawords.com/{}", util::encode(query))).send()?
        )?;
        let mut defs = MultiMap::new();
        let mut article = String::new();
        let word = page
            .find(Class("title-word"))
            .next()
            .ok_or_else(||ParseErr(err_msg("Missing title-word")))?
            .text()
            .trim()
            .to_owned();
        for node in page.find(Name("dd")) {
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
        Ok(stringify(&word, &defs))
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
            s.push_str(" ");
            s.push_str(&i.to_string());
            s.push_str(". ");
            s.push_str(v);
            i += 1;
        }
    }
    s
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search() {
        assert_eq!(Define::new().search("gulch", &reqwest::Client::new()).unwrap(), "\x02gulch:\x02 \x1d(noun)\x1d 1. A narrow V-shaped valley with a stream running through it. 2. A remote town or village, lacking in infrastructure and equipment.");
    }
    
    #[test]
    fn test_clean_content() {
        assert_eq!(
            Define::new().clean("°The name for the fifth letter of the Greek alphabet, ε or Ε, preceded by delta (Δ, δ) and followed by zeta (Ζ, ζ). °In IPA, the phonetic symbol that represents the ; represented in SAMPA as E."),
            "°The name for the fifth letter of the Greek alphabet, ε or Ε, preceded by delta and followed by zeta. °In IPA, the phonetic symbol that represents the .".to_owned()
        );
    }

    #[test]
    fn test_not_found() {
        assert!(Define::new().search("shisno", &reqwest::Client::new()).is_err());
    }
}
