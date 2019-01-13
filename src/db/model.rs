#![allow(proc_macro_derive_resolution_fallback)]
use std::time::SystemTime;
use crate::db::schema::*;
use crate::local::Local;

model!{Memo; DbMemo; "memo"; {
    channel: String,
    user:    String,
    message: String
}}
impl Local for Memo {
    fn channel(&self) -> String { self.channel.to_owned() }
    fn user(&self)    -> String { self.user.to_owned() }
}

model!{Reminder; DbReminder; "reminder"; {
    user:    String,
    when:    SystemTime,
    message: String
}}

model!{Seen; DbSeen; "seen"; {
    channel:     String,
    user:        String,
    first:       String,
    first_time:  SystemTime,
    latest:      String,
    latest_time: SystemTime,
    total:       i32
}}
impl Local for Seen {
    fn channel(&self) -> String { self.channel.to_owned() }
    fn user(&self)    -> String { self.user.to_owned() }
}

model!{Silence; DbSilence; "silence"; {
    channel: String,
    command: String
}}
impl Local for Silence {
    fn channel(&self) -> String { self.channel.to_owned() }
    fn user(&self)    -> String { self.command.to_owned() }
}

model!{Tell; DbTell; "tell"; {
    target:  String,
    sender:  String,
    time:    SystemTime,
    message: String
}}

#[table_name = "user"]
#[derive(Queryable, Insertable)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct User {
    pub nick:     String,
    pub auth:     i32,
    pub pronouns: Option<String>
}

#[table_name = "name_male"]
#[table_name = "name_female"]
#[table_name = "name_last"]
#[derive(Insertable, Queryable)]
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Name {
    pub name:        String,
    pub frequency:   i32,
    pub probability: f64
}
