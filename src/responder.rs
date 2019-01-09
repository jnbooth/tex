#![allow(dead_code)]

use irc::client::prelude::*;
use irc::error::IrcError;

use crate::color;
use crate::color::log;

pub trait Responder {
    fn action(&self, target: &str, msg: &str) -> Result<(), IrcError>;
    fn ban(&self, target: &str, source: &str, msg: &str) -> Result<(), IrcError>;
    fn privmsg(&self, target: &str, msg: &str) -> Result<(), IrcError>;
    fn reply(&self, target: &str, source: &str, msg: &str) -> Result<(), IrcError>;
    fn quit(&self, msg: &str) -> Result<(), IrcError>;
    fn warn(owner: &Option<String>, msg: &str) -> String;
}

impl Responder for IrcClient {
    fn action(&self, target: &str, msg: &str) -> Result<(), IrcError> {
        log(color::ECHO, &format!("> /me {}", msg));
        self.send_action(target, msg)
    }
    fn ban(&self, target: &str, source: &str, msg: &str) -> Result<(), IrcError> {
        log(color::WARN, &format!("Banning {} from {}: {}", source, target, msg));
        self.send_kick(target, source, msg)?;
        self.send_mode(source, &[Mode::Plus(ChannelMode::Ban, Some(target.to_owned()))])
    }
    fn privmsg(&self, target: &str, msg: &str) -> Result<(), IrcError> {
        log(color::ECHO, &format!("> {}", msg));
        self.send_privmsg(target, msg)
    }
    fn reply(&self, target: &str, source: &str, msg: &str) -> Result<(), IrcError> {
        self.privmsg(target, &format!("{}: {}", source, msg))
    }
    fn quit(&self, msg: &str) -> Result<(), IrcError> {
        self.send_quit(msg.to_owned())
    }
    fn warn(owner: &Option<String>, msg: &str) -> String {
        log(color::WARN, msg);
        match owner {
            None    => "Something went wrong.".to_owned(),
            Some(s) => format!("Something went wrong. Please let {} know.", s)
        }
    }
}

#[cfg(test)]
pub struct Debugger;

#[cfg(test)]
impl Responder for Debugger {
    fn action(&self, _: &str, msg: &str) -> Result<(), IrcError> {
        log(color::ECHO, &format!("> /me {}", msg));
        Ok(())
    }
    fn ban(&self, target: &str, source: &str, msg: &str) -> Result<(), IrcError> {
        log(color::WARN, &format!("Banning {} from {}: {}", source, target, msg));
        Ok(())
    }
    fn privmsg(&self, _: &str, msg: &str) -> Result<(), IrcError> {
        log(color::ECHO, &format!("> {}", msg));
        Ok(())
    }
    fn reply(&self, target: &str, source: &str, msg: &str) -> Result<(), IrcError> {
        self.privmsg(target, &format!("{}: {}", source, msg))
    }
    fn quit(&self, msg: &str) -> Result<(), IrcError> {
        log(color::ECHO, msg);
        Ok(())
    }
    fn warn(_: &Option<String>, msg: &str) -> String {
        panic!(msg.to_owned())
    }
}
