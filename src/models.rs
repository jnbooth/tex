#![allow(proc_macro_derive_resolution_fallback)]
use std::time::SystemTime;

#[derive(Queryable)]
pub struct Property {
    pub key: String,
    pub value: String
}

use super::schema::reminder;
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

use super::schema::user;
#[derive(Insertable, Queryable)]
#[table_name = "user"]
pub struct User {
    pub nick: String,
    pub auth: i32,
    pub pronouns: Option<String>
}
