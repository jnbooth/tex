#![allow(proc_macro_derive_resolution_fallback)]
use std::time::SystemTime;
use chrono::{DateTime, Utc};
use std::borrow::ToOwned;
use xmlrpc::Value;

use crate::db::*;
use crate::local::Local;

#[table_name = "memo"]
#[derive(Insertable, Queryable)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Memo {
    pub channel: String,
    pub user:    String,
    pub message: String
}
impl Local for Memo {
    fn channel(&self) -> String { self.channel.to_owned() }
    fn obj(&self)     -> String { self.user.to_owned() }
}

model!{Reminder; DbReminder; "reminder"; {
    user:    String,
    when:    SystemTime,
    message: String
}}
impl Default for Reminder {
    fn default() -> Self {
        Self { user: String::default(), when: SystemTime::now(), message: String::default() }
    }
}


#[table_name = "seen"]
#[derive(Insertable, Queryable)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Seen {
    pub channel:     String,
    pub user:        String,
    pub first:       String,
    pub first_time:  SystemTime,
    pub latest:      String,
    pub latest_time: SystemTime,
    pub total:       i32
}
impl Default for Seen {
    fn default() -> Self {
        Self {
            channel:     String::default(),
            user:        String::default(),
            first:       String::default(),
            first_time:  SystemTime::now(),
            latest:      String::default(),
            latest_time: SystemTime::now(),
            total:       i32::default()
        }
    }
}
impl Local for Seen {
    fn channel(&self) -> String { self.channel.to_owned() }
    fn obj(&self)     -> String { self.user.to_owned() }
}

#[table_name = "silence"]
#[derive(Insertable, Queryable, Default)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Silence {
    pub channel: String,
    pub command: String
}
impl Local for Silence {
    fn channel(&self) -> String { self.channel.to_owned() }
    fn obj(&self)     -> String { self.command.to_owned() }
}

#[table_name = "tag"]
#[derive(Insertable, Queryable, Default)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Tag {
    pub name: String,
    pub page: String
}

model!{Tell; DbTell; "tell"; {
    target:  String,
    sender:  String,
    time:    SystemTime,
    message: String
}}
impl Default for Tell {
    fn default() -> Self {
        Self { 
            target:  String::default(), 
            sender:  String::default(), 
            time:    SystemTime::now(),
            message: String::default()
            }
    }
}

#[table_name = "user"]
#[derive(Insertable, Queryable, Default)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct User {
    pub nick:     String,
    pub auth:     i32,
    pub pronouns: Option<String>
}

#[table_name = "name_male"]
#[table_name = "name_female"]
#[table_name = "name_last"]
#[derive(Insertable, Queryable, Default)]
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Name {
    pub name:        String,
    pub frequency:   i32,
    pub probability: f64
}

#[table_name = "attribution"]
#[derive(Insertable, Queryable)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Attribution {
    pub page: String,
    pub user: String,
    pub kind: String
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
impl Default for Page {
    fn default() -> Self {
        Self {
            fullname:   String::default(),
            created_at: Utc::now(),
            created_by: String::default(),
            rating:     i32::default(),
            title:      String::default()
        }
    }
}

impl Page {
    pub fn build(val: &Value) -> Option<Page> {
        let obj = val.as_struct()?;
        let created_at = DateTime
            ::parse_from_rfc3339(obj.get("created_at")?.as_str()?)
            .ok()?
            .with_timezone(&Utc);
        let created_by = obj.get("created_by")?.as_str()?.to_lowercase();
        let fullname = obj.get("fullname")?.as_str()?.to_owned();
        let rating = obj.get("rating")?.as_i32()?;
        let title = obj.get("title")?.as_str()?.to_owned();
        Some(Self { created_at, created_by, fullname, rating, title })
    }

    pub fn tagged<T: FromIterator<String>>(val: &Value) -> Option<(Self, T)> {
        let tags = val.as_struct()?.get("tags")?.as_array()?.into_iter()
            .filter_map(Value::as_str).map(ToOwned::to_owned).collect();
        Some((Page::build(val)?, tags))
    }
}

joinable!(tag -> page (page));
