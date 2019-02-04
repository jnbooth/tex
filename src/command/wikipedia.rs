use regex::Regex;
use serde_json::{Map, Value};

use super::*;
use crate::util;

const SEARCH_URL: &str = 
"https://en.wikipedia.org/w/api.php?format=json\
&formatversion=2&action=query&list=search&srlimit=1&srprop=&srsearch=";

const ENTRY_URL: &str = 
"https://en.wikipedia.org/w/api.php?format=json\
&action=query&prop=extracts|links&pllimit=100&exintro&explaintext&redirects=1&pageids=";

pub struct Wikipedia {
    parens: Regex
}

impl Command for Wikipedia {
    fn cmds(&self) -> Vec<String> {
        abbrev("wikipedia")
    }
    fn usage(&self) -> String { "<query>".to_owned() }
    fn fits(&self, size: usize) -> bool { size >= 1 }
    fn auth(&self) -> Auth { Anyone }

    fn run(&mut self, args: &[&str], _: &Context, db: &mut Db) -> Outcome {
        Ok(vec![Reply(self.search(&args.join(" "), &db.client)?)])
    }
}

impl Default for Wikipedia { fn default() -> Self { Self::new() } }

impl Wikipedia {
    #[inline]
    pub fn new() -> Self {
        Self {
            parens: Regex::new("\\s*\\([^()]+\\)").expect("Parens regex failed to compile")
        }
    }
    #[inline]
    fn clean(&self, s: &str) -> String {
        self.parens.replace_all(&s.replace("(listen)", ""), "").replace("  ", " ")
    }
    
    fn search(&self, query: &str, cli: &reqwest::Client) -> Result<String, Error> {
        let searches = serde_json::from_reader(
            cli.get(&format!("{}{}", SEARCH_URL, encode(query))).send()?
        )?;
        let page = parse_page(&searches)
            .ok_or_else(|| ParseErr(err_msg("Unable to parse results")))?;
        let entry = serde_json::from_reader(
            cli.get(&format!("{}{}", ENTRY_URL, encode(&page.to_string()))).send()?
        )?;
        self.get_entry(page, &entry)
            .ok_or_else(||ParseErr(err_msg("Unable to parse entry")))?
    }
  
    fn get_entry(&self, page: u64, json: &Value) -> Option<Result<String, Error>> {
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
        if top.ends_with(':') && top.contains("refer") {
            if let Some(disambig) = parse_disambig(title, result) {
                return Some(Err(Ambiguous(0, disambig)))
            }
        }
        Some( Ok(
            util::trim(&format!(
                "{} \x02{}\x02: {}", 
                format!("https://en.wikipedia.org/wiki/{}", encode(title)), 
                title, 
                self.clean(&extract.replace("\n", " "))
            ))
        ) )
    }
}

#[inline]
fn encode(s: &str) -> String {
    util::encode(&s.replace(" ", "_"))
}

fn parse_page(json: &Value) -> Option<u64> {
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

fn parse_link(json: &Value) -> Option<String> {
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

fn parse_disambig(title_up: &str, json: &Map<String, Value>) -> Option<Vec<String>> {
    let title = format!("{} (", title_up.to_lowercase());
    let links = json
        .get("links")?
        .as_array()?
        .into_iter()
        .filter_map(parse_link);
    let mut verbatim = links.clone()
        .filter(|x| x.to_lowercase().starts_with(&title))
        .peekable();
    if verbatim.peek().is_some() {
        Some(verbatim.collect())
    } else {
        Some(links.collect())
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn searches() {
        assert_eq!(Wikipedia::new().test_def("Monty Oum").unwrap(), "https://en.wikipedia.org/wiki/Monty_Oum \x02Monty Oum\x02: Monyreak \"Monty\" Oum was an American web-based animator and writer. A self-taught animator, he scripted and produced several crossover fighting video series, drawing the attention of internet production company Rooster Teeth, who hired him. […]");
    }

    #[test]
    fn disambiguates() {
        assert!(Wikipedia::new().test_def("Rock").is_err());
    }
}
