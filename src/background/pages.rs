use diesel::prelude::*;
use diesel::query_dsl::RunQueryDsl;
use diesel::pg::upsert::excluded;
use reqwest::Client;
use std::time::SystemTime;

use crate::IO;
use crate::db::Conn;
use crate::wikidot::Wikidot;
use crate::db::{attribution, page, tag};

pub fn update(cli: &Client, conn: &Conn, wiki: &Wikidot) -> IO<()> {
    let updated = SystemTime::now();
    let mut pages = Vec::new();
    let mut tags = Vec::new();
    let titles: Vec<String> = wiki.list(cli)?;
    wiki.walk(updated, &titles, cli, |page, mut pagetags| {
        pages.push(page);
        tags.append(&mut pagetags);
        Ok(())
    })?;
    for chunk in pages.chunks(10_000) {
        diesel::insert_into(page::table)
            .values(chunk)
            .on_conflict(page::id)
            .do_update()
            .set((
                page::created_at.eq(excluded(page::created_at)),
                page::created_by.eq(excluded(page::created_by)),
                page::rating.eq(excluded(page::rating)),
                page::title.eq(excluded(page::title)),
                page::updated.eq(excluded(page::updated))
            ))
            .execute(conn)?;
    }
    for chunk in tags.chunks(20_000) {
        diesel::insert_into(tag::table)
            .values(chunk)
            .on_conflict((tag::page_id, tag::name))
            .do_update()
            .set(tag::updated.eq(excluded(tag::updated)))
            .execute(conn)?;
    }
    diesel::delete(page::table.filter(page::updated.lt(updated))).execute(conn)?;
    diesel::delete(tag::table.filter(tag::updated.lt(updated))).execute(conn)?;
    diesel::delete(
        attribution::table.filter(attribution::page_id.ne_all(page::table.select(page::id)))
    ).execute(conn)?;
    Ok(())
}
