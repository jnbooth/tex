use diesel::prelude::*;
use diesel::query_dsl::RunQueryDsl;
use reqwest::Client;
use std::time::SystemTime;

use crate::IO;
use crate::db::{Conn, upsert};
use crate::wikidot::Wikidot;
use crate::db::{attribution, page, tag};


pub fn update(cli: &Client, conn: &Conn, wiki: &Wikidot) -> IO<()> {
    let updated = SystemTime::now();
    let mut pages = Vec::new();
    let mut tags = Vec::new();
    wiki.walk(updated, &wiki.list(cli)?, cli, |page, mut pagetags| {
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
                upsert(page::created_at),
                upsert(page::created_by),
                upsert(page::rating),
                upsert(page::title),
                upsert(page::updated)
            ))
            .execute(conn)?;
    }
    for chunk in tags.chunks(20_000) {
        diesel::insert_into(tag::table)
            .values(chunk)
            .on_conflict((tag::page_id, tag::name))
            .do_update()
            .set(upsert(tag::updated))
            .execute(conn)?;
    }
    diesel::delete(page::table.filter(page::updated.lt(updated))).execute(conn)?;
    diesel::delete(tag::table.filter(tag::updated.lt(updated))).execute(conn)?;
    diesel::delete(
        attribution::table.filter(attribution::page_id.ne_all(page::table.select(page::id)))
    ).execute(conn)?;
    Ok(())
}
