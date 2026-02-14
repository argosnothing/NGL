mod db;
mod providers;
mod registry;
mod schema;

use db::services::query_data;
use registry::ProviderRegistry;
use schema::{NGLDataKind, NGLRequest};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <search_term>", args[0]);
        std::process::exit(1);
    }

    let term = args[1].clone();

    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://ngl.db?mode=rwc".to_string());

    let db = db::establish_connection(&database_url).await?;

    println!("Database connected!");

    let request = NGLRequest {
        search_term: Some(term.clone()),
        providers: None,
        kinds: Some(vec![
            NGLDataKind::Function,
            // NGLDataKind::Example,
            NGLDataKind::Package,
            NGLDataKind::Option,
        ]),
    };

    ProviderRegistry::sync(&db, request.clone()).await?;
    println!("Sync complete!");

    println!("\nQuerying for '{}'...", term.clone());

    // Example NGL response
    let response = query_data(&db, &request).await?;
    println!("Response: {:#?}", response);

    Ok(())
}
