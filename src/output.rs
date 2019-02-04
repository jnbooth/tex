use irc::client::prelude::*;
use irc::error::IrcError;
use irc::client::data::user::AccessLevel;
use irc::client::data::user::AccessLevel::*;

use crate::logging::*;
use crate::Context;
use crate::auth::Auth;

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
    fn auth(&self, ctx: &Context) -> Auth;
    fn respond(&self, ctx: &Context, response: Response) -> Result<(), IrcError>;
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
    fn auth(&self, ctx: &Context) -> Auth {
        match access(&self, ctx) {
            Some(Owner)  => Auth::Op,
            Some(Admin)  => Auth::Op,
            Some(Oper)   => Auth::Op,
            Some(HalfOp) => Auth::HalfOp,
            Some(_)      => Auth::Anyone,
            None         => Auth::Anyone
        }
    }
    fn respond(&self, ctx: &Context, response: Response) -> Result<(), IrcError> {
        match response {
            Action(msg) => {        
                log(ECHO, &format!("{}| /me {}", ctx.since(), msg));
                self.send_action(&ctx.channel, msg)
            },
            Ban(msg) => {
                log(WARNING, &format!(
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
    fn auth(&self, _: &Context) -> Auth {
        Auth::Owner
    }
    fn respond(&self, ctx: &Context, response: Response) -> Result<(), IrcError> {
        match response {
            Action(msg) => {        
                log(ECHO, &format!("{}| /me {}", ctx.since(), msg));
                Ok(())
            },
            Ban(msg) => {
                log(WARNING, &format!(
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
