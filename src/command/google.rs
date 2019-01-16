use serde_json::Value;

use super::*;
use crate::{Api, util};

pub struct Google {
    api: Api,
    img: bool
}

impl<O: Output + 'static> Command<O> for Google {
    fn cmds(&self) -> Vec<String> {
        if self.img { own(&["gis"]) } else { abbrev("google") }
    }
    fn usage(&self) -> String { "<query>".to_owned() }
    fn fits(&self, size: usize) -> bool { size > 0 }
    fn auth(&self) -> i32 { 0 }
    fn reload(&mut self, _: &mut Db) -> Outcome<()> { Ok(()) }

    fn run(&mut self, args: &[&str], irc: &O, ctx: &Context, db: &mut Db) -> Outcome<()> {
        irc.reply(ctx, &self.search(&args.join(" "), &db.client)?)?;
        Ok(())
    }
}

impl Google {
    pub fn build(img: bool) -> Option<Self> {
        Some(Self {
            api: env::api("GOOGLE", "CUSTOMENGINE", "KEY")?,
            img
        })
    }
    
    fn parse(&self, json: &Value) -> Option<String> {
        let obj = json
            .as_object()?
            .get("items")?
            .as_array()?
            .get(0)?
            .as_object()?;
        let get = |key| Some(obj.get(key)?.as_str()?.replace("\"", ""));
        let title = ellipses(&get("title")?);
        let link = get("link")?;
        if self.img {
            Some(format!("{} \x02{}\x02", link, title))
        } else {    
            let snippet = ellipses(&get("snippet")?.replace("\n", ""));
            Some(format!("{} \x02{}\x02: {}", link, title, snippet))
        }
    }

    fn search(&self, query: &str, cli: &reqwest::Client) -> Outcome<String> {
        let search_res = cli.get(&format!(
            "https://www.googleapis.com/customsearch/v1?key={}&cx={}&q={}&alt=json{}",
            self.api.key, self.api.user, util::encode(query), 
            if self.img { "&searchType=image" } else { "" }
        )).send()?;
        self.parse(&serde_json::from_reader(search_res)?)
            .ok_or_else(||ParseErr(err_msg("Unable to parse results")))
    }
}

fn ellipses(s: &str) -> String {
    if s.ends_with("...") {
        let mut dots = s.to_owned();
        dots.pop();
        dots.pop();
        dots.pop();
        dots.push_str("[…]");
        dots
    } else {
        s.to_owned()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn new(img: bool) -> Google {
        env::load();
        Google::build(img).expect("Error initializing Google API")
    }

    #[test]
    fn test_ellipses() {
        assert_eq!(ellipses("...."), ".[…]");
    }

    #[test] #[ignore]
    fn test_search() {
        new(false).search("puma", &reqwest::Client::new()).unwrap();
    }

    #[test] #[ignore]
    fn test_search_fail() {
        assert!(new(false).search("!@#$", &reqwest::Client::new()).is_err());
    }
    
    #[test] #[ignore]
    fn test_image_search() {
        new(true).search("puma", &reqwest::Client::new()).unwrap();
    }

    #[test] #[ignore]
    fn test_image_search_fail() {
        assert!(new(true).search("!@#$", &reqwest::Client::new()).is_err());
    }
}
