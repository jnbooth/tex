#![allow(proc_macro_derive_resolution_fallback)]
use std::time::SystemTime;
use chrono::{DateTime, Utc};
use std::borrow::ToOwned;
use std::hash::{Hash, Hasher};
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
    pub user:    String,
    pub when:    SystemTime,
    pub message: String
}}
impl Default for Reminder {
    fn default() -> Self {
        Self { user: String::default(), when: SystemTime::now(), message: String::default() }
    }
}


#[table_name = "seen"]
#[derive(Insertable, Queryable)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SeenInsert {
    pub channel: String,
    pub user:    String,
    pub first:   String,
    pub latest:  String
}

#[derive(Queryable)]
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

model!{Tell; DbTell; "tell"; {
    pub target:  String,
    pub sender:  String,
    pub time:    SystemTime,
    pub message: String
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

#[table_name = "namegen"]
#[derive(Insertable, Queryable, Default)]
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct NameGen {
    pub kind:      String,
    pub name:      String,
    pub frequency: i32
}

#[table_name = "page"]
#[derive(Identifiable, Insertable, Queryable)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Page {
    pub id:         String,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    pub rating:     i32,
    pub title:      String,
    pub updated:    SystemTime
}

impl Default for Page {
    fn default() -> Self {
        Self {
            id:         String::default(),
            created_at: Utc::now(),
            created_by: String::default(),
            rating:     i32::default(),
            title:      String::default(),
            updated:    SystemTime::now()
        }
    }
}

impl Page {
    pub fn build(val: &Value, updated: SystemTime) -> Option<Page> {
        let obj = val.as_struct()?;
        let created_at = DateTime::parse_from_rfc3339(obj.get("created_at")?.as_str()?)
            .ok()?
            .with_timezone(&Utc);
        let created_by = obj.get("created_by")?.as_str()?.to_lowercase();
        let id = obj.get("fullname")?.as_str()?.to_owned();
        let rating = obj.get("rating")?.as_i32()?;
        let title = obj.get("title")?.as_str()?.to_owned();
        Some(Self { created_at, created_by, id, rating, title, updated })
    }

    pub fn tagged<T: FromIterator<Tag>>(val: &Value, updated: SystemTime) -> Option<(Self, T)> {
        let page = Page::build(val, updated)?;
        let tags = val
            .as_struct()?
            .get("tags")?
            .as_array()?
            .into_iter()
            .filter_map(Value::as_str)
            .map(|tag| Tag { 
                name: tag.to_owned(), 
                page_id: page.id.to_owned(), 
                updated
            })
            .collect();
        Some((page, tags))
    }
}

#[belongs_to(Page)]
#[table_name = "attribution"]
#[derive(Associations, Insertable, Queryable)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Attribution {
    pub page_id: String,
    pub user:    String,
    pub kind:    String
}


#[belongs_to(Page)]
#[table_name = "tag"]
#[derive(Associations, Insertable, Queryable)]
#[derive(Debug, Clone, PartialOrd, Ord, Eq)]
pub struct Tag {
    pub page_id: String,
    pub name:    String,
    pub updated: SystemTime
}

impl PartialEq for Tag {
    fn eq(&self, other: &Tag) -> bool {
        self.page_id == other.page_id && self.name == other.name
    }
}
impl Hash for Tag {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.page_id.hash(state);
        self.name.hash(state);
    }
}
