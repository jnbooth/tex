#![allow(proc_macro_derive_resolution_fallback)]
use std::time::SystemTime;
use crate::db::schema::*;

model!(Reminder; DbReminder; "reminder"; {
    nick:    String,
    when:    SystemTime,
    message: String
});

model!(Seen; DbSeen; "seen"; {
    channel:     String,
    nick:        String,
    first:       String,
    first_time:  SystemTime,
    latest:      String,
    latest_time: SystemTime,
    total:       i32
});

model!(Silence; DbSilence; "silence"; {
    channel: String,
    command: String
});

model!(Tell; DbTell; "tell"; {
    target:  String,
    sender:  String,
    time:    SystemTime,
    message: String
});

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
