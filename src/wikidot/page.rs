use chrono::{DateTime, FixedOffset};
use std::borrow::ToOwned;
use xmlrpc::Value;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Page {
    pub created_at: DateTime<FixedOffset>,
    pub created_by: String,
    pub fullname: String,
    pub rating: i32,
    pub tags: Vec<String>,
    pub title: String
}

impl Page {
    pub fn new(val: &Value) -> Option<Page> {
        let obj = val.as_struct()?;
        let created_at = DateTime::parse_from_rfc3339(obj.get("created_at")?.as_str()?).ok()?;
        let created_by = obj.get("created_by")?.as_str()?.to_owned();
        let fullname = obj.get("fullname")?.as_str()?.to_owned();
        let rating = obj.get("rating")?.as_i32()?;
        let title = obj.get("title")?.as_str()?.to_owned();
        let tags = obj.get("tags")?.as_array()?.into_iter()
            .filter_map(Value::as_str).map(ToOwned::to_owned).collect();
        Some(Page { created_at, created_by, fullname, rating, tags, title })
    }
}
