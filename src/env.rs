use irc::client::prelude::Config;

use crate::Api;

#[inline]
pub fn load() {
    match dotenv::dotenv() {
        Err(dotenv::Error::Io(_)) => (),
        Ok(_)                     => (),
        _                         => panic!("Error loading .env")
    }
}

#[inline]
pub fn get(var: &str) -> String {
    std::env::var(var)
        .unwrap_or_else(|_|
            panic!(format!("{} must be defined in .env or as an environment variable", var))
        )
}

#[inline]
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
        server:       Some(get("IRC_SERVER")),
        nickname:     Some(get("IRC_NICK")),
        password:     Some(get("IRC_PASSWORD")),
        channels:     Some(get("AUTOJOIN").split(',').map(|x| format!("#{}", x)).collect()),
        should_ghost: Some(true),
        #[cfg(test)]
        use_mock_connection: Some(true),
        ..Config::default()
    }
}
