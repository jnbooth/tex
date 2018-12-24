use diesel::prelude::*;
use diesel::pg::PgConnection;
use dotenv::dotenv;
use std::env;
use std::collections::HashMap;
use std::iter::*;

use super::models;
use super::schema;

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

pub fn load_properties(conn: &PgConnection) -> HashMap<String, String> {
    HashMap::from_iter(
        schema::property::table.load(conn)
            .expect("Error loading properties")
            .into_iter()
            .map(|x: models::Property| (x.key, x.value))
    )
}
