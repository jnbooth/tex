use irc::client::prelude::*;
use irc::error::IrcError;

use crate::logging::*;
use crate::Context;

pub trait Output {
    fn action(&self, ctx: &Context, msg: &str) -> Result<(), IrcError>;
    fn ban(&self, ctx: &Context, msg: &str) -> Result<(), IrcError>;
    fn msg(&self, ctx: &Context, msg: &str) -> Result<(), IrcError>;
    fn pm(&self, ctx: &Context, msg: &str) -> Result<(), IrcError>;
    fn reply(&self, ctx: &Context, msg: &str) -> Result<(), IrcError>;
    fn quit(&self, msg: &str) -> Result<(), IrcError>;
    fn warn(owner: &Option<String>, msg: &str) -> String;
}

impl Output for IrcClient {
    fn action(&self, ctx: &Context, msg: &str) -> Result<(), IrcError> {
        log(ECHO, &format!("{}| /me {}", ctx.since(), msg));
        self.send_action(&ctx.channel, msg)
    }
    fn ban(&self, ctx: &Context, msg: &str) -> Result<(), IrcError> {
        log(WARN, &format!(
            "{}! Banning {} from {}: {}", ctx.since(), ctx.nick, ctx.channel, msg
        ));
        self.send_kick(&ctx.channel, &ctx.nick, msg)?;
        self.send_mode(&ctx.nick, &[Mode::Plus(ChannelMode::Ban, Some(ctx.channel.to_owned()))])
    }
    fn msg(&self, ctx: &Context, msg: &str) -> Result<(), IrcError> {
        if ctx.channel == ctx.user {
            self.pm(ctx, msg)
        } else {
            log(ECHO, &format!("{}| {}", ctx.since(), msg));
            self.send_notice(&ctx.channel, msg)
        }
    }
    fn pm(&self, ctx: &Context, msg: &str) -> Result<(), IrcError> {
        log(ECHO, &format!("{}@ {}", ctx.since(), msg));
        self.send_privmsg(&ctx.user, msg)
    }
    fn reply(&self, ctx: &Context, msg: &str) -> Result<(), IrcError> {
        self.msg(&ctx, &format!("{}: {}", ctx.nick, msg))
    }
    fn quit(&self, msg: &str) -> Result<(), IrcError> {
        self.send_quit(msg.to_owned())
    }
    fn warn(owner: &Option<String>, msg: &str) -> String {
        log(WARN, msg);
        match owner {
            None    => "Something went wrong.".to_owned(),
            Some(s) => format!("Something went wrong. Please let {} know.", s)
        }
    }
}

pub struct Offline;

impl Output for Offline {
    fn action(&self, ctx: &Context, msg: &str) -> Result<(), IrcError> {
        log(ECHO, &format!("{}| /me {}", ctx.since(), msg));
        Ok(())
    }
    fn ban(&self, ctx: &Context, msg: &str) -> Result<(), IrcError> {
        log(WARN, &format!(
            "{}! Banning {} from {}: {}", ctx.since(), ctx.nick, ctx.channel, msg
        ));
        Ok(())
    }
    fn msg(&self, ctx: &Context, msg: &str) -> Result<(), IrcError> {
        if ctx.channel == ctx.user {
            self.pm(ctx, msg)
        } else {
            log(ECHO, &format!("{}| {}", ctx.since(), msg));
            Ok(())
        }
    }
    fn pm(&self, ctx: &Context, msg: &str) -> Result<(), IrcError> {
        log(ECHO, &format!("{}@ {}", ctx.since(), msg));
        Ok(())
    }
    fn reply(&self, ctx: &Context, msg: &str) -> Result<(), IrcError> {
        self.msg(&ctx, &format!("{}: {}", ctx.nick, msg))
    }
    fn quit(&self, msg: &str) -> Result<(), IrcError> {
        panic!(msg.to_owned())
    }
    fn warn(owner: &Option<String>, msg: &str) -> String {
        log(WARN, msg);
        match owner {
            None    => "Something went wrong.".to_owned(),
            Some(s) => format!("Something went wrong. Please let {} know.", s)
        }
    }
}
