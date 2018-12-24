#![allow(proc_macro_derive_resolution_fallback)]

#[derive(Queryable)]
pub struct Property {
    pub key: String,
    pub value: String
}

use super::schema::user;
#[derive(Insertable, Queryable)]
#[table_name = "user"]
pub struct User {
    pub nick: String,
    pub auth: i32,
    pub pronouns: Option<String>
}
