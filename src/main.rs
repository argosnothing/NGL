mod db;
mod providers;
mod schema;

use providers::{Provider, noogle::Noogle};
use schema::{NGLDataKind, NGLRequest};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://ngl.db?mode=rwc".to_string());

    let db = db::establish_connection(&database_url).await?;

    println!("Database connected!");

    let request = NGLRequest {
        search_term: None,
        providers: None,
        kinds: Some(vec![NGLDataKind::Function, NGLDataKind::Example]),
    };

    println!("Syncing Noogle data...");
    Noogle::sync(&db, request).await?;
    println!("Sync complete!");

    Ok(())
}
