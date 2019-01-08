use reqwest::Client;
use serde_json::Value;

use crate::{Api, IO, util};

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

fn parse(json: &Value, image: bool) -> Option<String> {
    let obj = json
        .as_object()?
        .get("items")?
        .as_array()?
        .get(0)?
        .as_object()?;
    let get = |key| Some(obj.get(key)?.as_str()?.replace("\"", ""));
    let title = ellipses(&get("title")?);
    let link = get("link")?;
    if image {
        Some(format!("{} - \x02{}\x02", link, title))
    } else {    
        let snippet = ellipses(&get("snippet")?.replace("\n", ""));
        Some(format!("{} - \x02{}\x02: {}", link, title, snippet))
    }
}

fn either_search(api: &Api, client: &Client, query: &str, image: bool) -> IO<String> {
    let search_res = client.get(&format!(
        "https://www.googleapis.com/customsearch/v1?key={}&cx={}&q={}&alt=json{}",
        api.key, api.user, util::encode(query), if image { "&searchType=image" } else { "" }
    )).send()?;
    parse(&serde_json::from_reader(search_res)?, image)
        .ok_or(failure::err_msg("Unable to parse Google results."))
}

pub fn search(api: &Api, client: &Client, query: &str) -> IO<String> {
    either_search(api, client, query, false)
}

pub fn search_image(api: &Api, client: &Client, query: &str) -> IO<String> {
    either_search(api, client, query, true)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::db::Apis;
    use dotenv::dotenv;

    #[test]
    fn test_ellipses() {
        assert_eq!(ellipses("...."), ".[…]");
    }

    #[test]
    fn test_search() {
        dotenv().unwrap();
        search(&Apis::new().google.unwrap(), &Client::new(), "puma").unwrap();
    }

    #[test]
    fn test_search_fail() {
        dotenv().unwrap();
        assert!(search(&Apis::new().google.unwrap(), &Client::new(), "!@#$").is_err());
    }
    
    #[test]
    fn test_search_image() {
        dotenv().unwrap();
        search_image(&Apis::new().google.unwrap(), &Client::new(), "puma").unwrap();
    }

    #[test]
    fn test_search_image_fail() {
        dotenv().unwrap();
        assert!(search_image(&Apis::new().google.unwrap(), &Client::new(), "!@#$").is_err());
    }
}
