#![allow(proc_macro_derive_resolution_fallback)]
use std::time::SystemTime;
use chrono::{DateTime, Utc};
use std::borrow::ToOwned;
use xmlrpc::Value;

use crate::db::*;
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

model!{Tag; DbTag; "tag"; {
    name:       String,
    page:       String
}}

model!{Tell; DbTell; "tell"; {
    target:  String,
    sender:  String,
    time:    SystemTime,
    message: String
}}

#[table_name = "user"]
#[derive(Insertable, Queryable)]
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


#[table_name = "page"]
#[derive(Insertable, Queryable)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Page {
    pub fullname: String,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    pub rating: i32,
    pub title: String
}

impl Page {
    pub fn new(val: &Value) -> Option<Page> {
        Some(Page::tagged(val)?.0)
    }

    pub fn tagged(val: &Value) -> Option<(Page, Vec<String>)> {
        let obj = val.as_struct()?;
        let created_at = DateTime
            ::parse_from_rfc3339(obj.get("created_at")?.as_str()?)
            .ok()?
            .with_timezone(&Utc);
        let created_by = obj.get("created_by")?.as_str()?.to_owned();
        let fullname = obj.get("fullname")?.as_str()?.to_owned();
        let rating = obj.get("rating")?.as_i32()?;
        let title = obj.get("title")?.as_str()?.to_owned();
        let tags = obj.get("tags")?.as_array()?.into_iter()
            .filter_map(Value::as_str).map(ToOwned::to_owned).collect();
        Some((Page { created_at, created_by, fullname, rating, title }, tags))
    }
}

joinable!(tag -> page (page));
