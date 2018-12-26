use irc::client::prelude::*;
use irc::error::IrcError;

use super::super::color;
use super::super::color::log;

pub trait Responder {
    fn action(&self, target: &str, msg: &str) -> Result<(), IrcError>;
    fn privmsg(&self, target: &str, msg: &str) -> Result<(), IrcError>;
    fn reply(&self, target: &str, source: &str, msg: &str) -> Result<(), IrcError>;
    fn quit(&self, msg: &str) -> Result<(), IrcError>;
}

impl Responder for IrcClient {
    fn action(&self, target: &str, msg: &str) -> Result<(), IrcError> {
        log(color::ECHO, &format!("> /me {}", msg));
        self.send_action(target, msg)
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
}

pub struct _Debugger { }

impl Responder for _Debugger {
    fn action(&self, _: &str, msg: &str) -> Result<(), IrcError> {
        log(color::ECHO, &format!("> /me {}", msg));
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
}
