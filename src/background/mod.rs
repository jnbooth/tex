use diesel::prelude::*;
use diesel::query_dsl::RunQueryDsl;
use reqwest::Client;
use std::thread;
use std::time::{Duration, Instant};

use crate::IO;
use crate::db::{Conn, Db, Pool, timer};
use crate::wikidot::Wikidot;
use crate::logging::*;

mod attributions;
mod pages;
pub mod diff;
pub mod titles;

use self::diff::Diff;
use self::titles::TitlesDiff;

pub fn spawn(pool: Pool, db: &mut Db) -> IO<()> {
    thread("attributions", pool.clone(), attributions::update);
    thread("pages", pool.clone(), pages::update);

    let (mut titles, titles_r) = TitlesDiff::build(&db.client)?;
    db.titles   = titles.cache().clone().into_iter().collect();
    db.titles_r = Some(titles_r);
    thread("titles", pool, move |cli,_,_| titles.diff(cli));

    Ok(())
}

fn benchmark(ins: Instant) -> u64 {
    let dur = ins.elapsed();
    dur.as_secs() * 1000 + u64::from(dur.subsec_millis())
}

fn thread<F>(label: &'static str, pool: Pool, mut f: F) 
where F: Send + 'static + FnMut(&Client, &Conn, &Wikidot) -> IO<()> {
    let lower = label .to_lowercase();
    let missing_timer = format!("Missing timer: {}", label);
    let client = Client::new();
    let wiki = Wikidot::new();
    thread::spawn(move || loop {
        let now = Instant::now();
        let conn = pool.get().expect("Failed to get connection from database pool");
        f(&client, &conn, &wiki).log(trace!());
        log(INFO, &format!("Scanned {} in {}ms.", label, benchmark(now)));
        let minutes: i32 = timer::table
            .filter(timer::name.eq(&lower))
            .select(timer::minutes)
            .first(&conn)
            .expect(&missing_timer);
        thread::sleep(Duration::from_secs(60 * minutes as u64));
    });
}
