use serde_json::Value;

use super::*;
use crate::{Api, util};

pub struct Google {
    api: Api,
    img: bool
}

impl Command for Google {
    fn cmds(&self) -> Vec<String> {
        if self.img { own(&["gis"]) } else { abbrev("google") }
    }
    fn usage(&self) -> String { "<query>".to_owned() }
    fn fits(&self, size: usize) -> bool { size >= 1 }
    fn auth(&self) -> Auth { Anyone }

    fn run(&mut self, args: &[&str], _: &Context, db: &mut Db) -> Outcome {
        Ok(vec![Reply(self.search(&args.join(" "), &db.client)?)])
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

    fn search(&self, query: &str, cli: &reqwest::Client) -> Result<String, Error> {
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
    fn condenses_ellipses() {
        assert_eq!(ellipses("...."), ".[…]");
    }

    #[test]
    fn searches() {
        new(false).test_def("puma").unwrap();
    }

    #[test]
    fn not_found() {
        assert!(new(false).test_def("!@#$").is_err());
    }
    
    #[test]
    fn image_searches() {
        new(true).test_def("puma").unwrap();
    }

    #[test]
    fn image_not_found() {
        assert!(new(true).test_def("!@#$").is_err());
    }
}
