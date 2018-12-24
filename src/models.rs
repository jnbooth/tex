#![allow(proc_macro_derive_resolution_fallback)]
#[derive(Queryable)]
pub struct Property {
    pub key: String,
    pub value: String
}
