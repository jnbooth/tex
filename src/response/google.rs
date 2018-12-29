use serde_json::Value;

use super::super::{Api, IO, encode};

fn parse(json: &Value, image: bool) -> Option<String> {
    let obj = json
        .as_object()?
        .get("items")?
        .as_array()?
        .get(0)?
        .as_object()?;
    let get = |key| Some(obj.get(key)?.as_str()?.replace("\"", ""));
    let title = get("title")?;
    let link = get("link")?;
    if image {
        Some(format!("{} - \x02{}\x02", link, title))
    } else {    
        let mut snippet = get("snippet")?.replace("\n", "");
        if snippet.ends_with("...") {
            snippet = snippet[..snippet.len()-3].to_owned();
            snippet.push_str("[…]");
        }
        Some(format!("{} - \x02{}\x02: {}", link, title, snippet))
    }
}

fn either_search(api: &Api, client: &reqwest::Client, query: &str, image: bool) 
-> IO<String> {
    let search_res = client.get(&format!(
        "https://www.googleapis.com/customsearch/v1?key={}&cx={}&q={}&alt=json{}",
        api.key, api.user, encode(query), if image { "&searchType=image" } else { "" }
    )).send()?;
    let search_json = serde_json::from_reader(search_res)?;
    parse(&search_json, image).ok_or(failure::err_msg("Unable to parse Google results."))
}

pub fn search(api: &Api, client: &reqwest::Client, query: &str) -> IO<String> {
    either_search(api, client, query, false)
}

pub fn search_image(api: &Api, client: &reqwest::Client, query: &str) -> IO<String> {
    either_search(api, client, query, true)
}
