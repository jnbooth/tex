use irc::client::prelude::*;
use std::time::Instant;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Context {
    pub channel: String,
    pub nick:    String,
    pub host:    String,
    pub user:    String,
    pub time:    Instant
}

impl Context {
    pub fn build(message: Message) -> Option<Context> {
        let channel = message.response_target()?.to_lowercase();
        let prefix  = message.prefix?.to_owned();
        let nick    = prefix.split('!').next()?.to_owned();
        let host    = prefix.split('@').last()?.to_owned();
        let user    = nick.to_lowercase();
        let time    = Instant::now();

        Some(Self { channel, nick, host, user, time })
    }
    pub fn since(&self) -> String {
        let dur = self.time.elapsed();
        format!("{}.{:02}s ", dur.as_secs(), dur.subsec_millis() / 10)
    }
    #[cfg(test)]
    pub fn mock(channel: &str, nick: &str) -> Self {
        Context { 
             channel: channel.to_lowercase(),
             nick:    nick.to_owned(),
             host:    String::new(),
             user:    nick.to_lowercase(),
             time:    Instant::now()
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
             time:    Instant::now()
        }
    }
}
