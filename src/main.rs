mod db;
mod providers;
mod schema;

use chrono::{Datelike, Local, NaiveDate};
use providers::Provider;
use providers::noogle::Noogle;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://ngl.db?mode=rwc".to_string());

    let db = db::establish_connection(&database_url).await?;

    println!("Database connected!");

    let i = 3;
    let _response = Noogle::pull_data().await;
    let p = _response;
    let o = &i;

    Ok(())
}
