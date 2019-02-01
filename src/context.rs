use irc::client::prelude::*;
use std::time::SystemTime;

use crate::db::Db;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Context {
    pub channel: String,
    pub nick:    String,
    pub host:    String,
    pub user:    String,
    pub auth:    i32,
    pub time:    SystemTime
}

impl Context {
    pub fn build(db: &Db, message: Message) -> Option<Context> {
        let channel = message.response_target()?.to_lowercase();
        let prefix  = message.prefix?.to_owned();
        let nick    = prefix.split('!').next()?.to_owned();
        let host    = prefix.split('@').last()?.to_owned();
        let user    = nick.to_lowercase();
        let auth    = db.auth(&user);
        let time    = SystemTime::now();

        Some(Self { channel, nick, host, user, auth, time })
    }
    pub fn since(&self) -> String {
        match self.time.elapsed() {
            Err(_) => "now ".to_owned(),
            Ok(x)  => format!("{}.{:02}s ", x.as_secs(), x.subsec_millis() / 10)
        }
    }
    #[cfg(test)]
    pub fn mock(channel: &str, nick: &str) -> Self {
        Context { 
             channel: channel.to_lowercase(),
             nick:    nick.to_owned(),
             host:    String::new(),
             user:    nick.to_lowercase(),
             auth:    0,
             time:    SystemTime::now()
        }
    }
}
#[cfg(test)]
impl Default for Context {
    fn default() -> Self {
        Context { 
             channel: String::default(),
             nick:    String::default(),
             host:    String::default(),
             user:    String::default(),
             auth:    i32::default(),
             time:    SystemTime::now()
        }
    }
}
