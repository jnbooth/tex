use irc::client::prelude::Config;

use crate::Api;

pub fn load() {
    dotenv::dotenv().expect("Error loading .env");
}

pub fn get(var: &str) -> String {
    std::env::var(var).expect(&format!("{} must be set in .env", var))
}

pub fn opt(var: &str) -> Option<String> {
    let res = std::env::var(var).ok()?.trim().to_owned();
    if res.is_empty() { None } else { Some(res) }
}

pub fn api(prefix: &str, user: &str, key: &str) -> Option<Api> {
    Some(Api { 
        user: opt(&format!("{}_{}", prefix, user))?, 
        key:  opt(&format!("{}_{}", prefix, key))? 
    })
}

pub fn irc() -> Config {
    Config {
        server:   Some(get("IRC_SERVER")),
        nickname: Some(get("IRC_NICK")),
        password: Some(get("IRC_PASSWORD")),
        channels: Some(get("AUTOJOIN").split(',').map(|x| format!("#{}", x)).collect()),
        ..Config::default()
    }
}
