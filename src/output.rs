use irc::client::prelude::*;
use irc::error::IrcError;
use irc::client::data::user::AccessLevel;
use irc::client::data::user::AccessLevel::*;

use crate::logging::*;
use crate::Context;

use self::Response::*;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Response {
    Action(String),
    Ban(String),
    Message(String),
    Quit(String),
    Reply(String)
}

#[cfg(test)]
impl Response {
    pub fn text(&self) -> &str {
        match self {
            Action(s) => s,
            Ban(s) => s,
            Message(s) => s,
            Quit(s) => s,
            Reply(s) => s
        }
    }
}

pub trait Output {
    fn auth(&self, ctx: &Context) -> u8;
    fn respond(&self, ctx: &Context, response: Response) -> Result<(), IrcError>;
    fn warn(owner: &Option<String>, msg: &str) -> String {
        log(WARN, msg);
        match owner {
            None    => "Something went wrong.".to_owned(),
            Some(s) => format!("Something went wrong. Please let {} know.", s)
        }
    }
}

fn access(irc: &IrcClient, ctx: &Context) -> Option<AccessLevel> {
    Some(irc
        .list_users(&ctx.channel)?
        .into_iter()
        .find(|x| x.get_nickname() == ctx.nick)?
        .highest_access_level()
    )
}

impl Output for IrcClient {
    fn auth(&self, ctx: &Context) -> u8 {
        match access(&self, ctx) {
            None         => 0,
            Some(Owner)  => 3,
            Some(Admin)  => 3,
            Some(Oper)   => 3,
            Some(HalfOp) => 2,
            Some(_)      => 1
        }
    }
    fn respond(&self, ctx: &Context, response: Response) -> Result<(), IrcError> {
        match response {
            Action(msg) => {        
                log(ECHO, &format!("{}| /me {}", ctx.since(), msg));
                self.send_action(&ctx.channel, msg)
            },
            Ban(msg) => {
                log(WARN, &format!(
                    "{}! Banning {} from {}: {}", ctx.since(), ctx.nick, ctx.channel, msg
                ));
                self.send_kick(&ctx.channel, &ctx.nick, msg)?;
                self.send_mode(&ctx.nick, 
                    &[Mode::Plus(ChannelMode::Ban, Some(ctx.channel.to_owned()))]
                )
            },
            Message(msg) => {
                log(ECHO, &format!("{}@ {}", ctx.since(), msg));
                self.send_privmsg(&ctx.user, msg)
            },
            Quit(msg) => self.send_quit(msg),
            Reply(msg) => {
                let reply = format!("{}: {}", ctx.nick, msg);
                if ctx.channel == ctx.user {
                    self.respond(ctx, Message(reply))
                } else {
                    log(ECHO, &format!("{}| {}", ctx.since(), reply));
                    self.send_notice(&ctx.channel, reply)
                }
            }
        }
    }
}

pub struct Offline;

impl Output for Offline {
    fn auth(&self, _: &Context) -> u8 {
        4
    }
    fn respond(&self, ctx: &Context, response: Response) -> Result<(), IrcError> {
        match response {
            Action(msg) => {        
                log(ECHO, &format!("{}| /me {}", ctx.since(), msg));
                Ok(())
            },
            Ban(msg) => {
                log(WARN, &format!(
                    "{}! Banning {} from {}: {}", ctx.since(), ctx.nick, ctx.channel, msg
                ));
                Ok(())
            },
            Message(msg) => {
                log(ECHO, &format!("{}@ {}", ctx.since(), msg));
                Ok(())
            },
            Quit(msg) => panic!(msg),
            Reply(msg) => {
                let reply = format!("{}: {}", ctx.nick, msg);
                if ctx.channel == ctx.user {
                    self.respond(ctx, Message(reply))
                } else {
                    log(ECHO, &format!("{}| {}", ctx.since(), reply));
                    Ok(())
                }
            }
        }
    }
}
