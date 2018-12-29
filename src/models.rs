#![allow(proc_macro_derive_resolution_fallback)]
use std::time::SystemTime;
use super::schema::*;

#[macro_use]
mod model_macro;

#[derive(Insertable, Queryable)]
#[table_name = "property"]
pub struct Property {
    pub key:   String,
    pub value: String
}

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

#[derive(Insertable, Queryable)]
#[table_name = "user"]
pub struct User {
    pub nick:     String,
    pub auth:     i32,
    pub pronouns: Option<String>
}
