mod db;
mod providers;
mod registry;
mod schema;

use db::services::query_data;
use registry::ProviderRegistry;
use schema::{NGLDataKind, NGLRequest};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://ngl.db?mode=rwc".to_string());

    let db = db::establish_connection(&database_url).await?;

    println!("Database connected!");

    let term = "lib.optionalAttrs".to_string();
    // Example request for data containing term
    // on the function documentation
    let request = NGLRequest {
        search_term: Some(term.clone()),
        providers: None,
        kinds: Some(vec![NGLDataKind::Function, NGLDataKind::Example]),
    };

    println!("Syncing data...");
    ProviderRegistry::sync(&db, request.clone()).await?;
    println!("Sync complete!");

    println!("\nQuerying for '{}'...", term.clone());

    // Example NGL response
    let response = query_data(&db, &request).await?;
    println!("Response: {:#?}", response);

    Ok(())
}
