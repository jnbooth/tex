#![allow(proc_macro_derive_resolution_fallback)]
use std::time::SystemTime;
use super::schema::*;

#[derive(Queryable)]
pub struct Property {
    pub key: String,
    pub value: String
}

#[derive(Insertable)]
#[table_name = "reminder"]
pub struct DbReminder {
    pub nick: String,
    pub when: SystemTime,
    pub message: String
}
#[derive(Queryable)]
pub struct Reminder {
    _id: i32,
    pub nick: String,
    pub when: SystemTime,
    pub message: String
}

#[derive(Insertable, Queryable)]
#[table_name = "seen"]
pub struct Seen {
    pub nick: String,
    pub first: String,
    pub first_time: SystemTime,
    pub latest: String,
    pub latest_time: SystemTime,
    pub total: i32
}

#[derive(Insertable)]
#[table_name = "silence"]
pub struct DbSilence {
    pub command: String,
    pub channel: String
}
#[derive(Queryable)]
pub struct Silence {
    _id: i32,
    pub command: String,
    pub channel: String
}

#[derive(Insertable, Queryable)]
#[table_name = "user"]
pub struct User {
    pub nick: String,
    pub auth: i32,
    pub pronouns: Option<String>
}
