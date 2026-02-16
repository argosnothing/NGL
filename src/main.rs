mod cli;
mod db;
mod providers;
mod registry;
mod schema;
mod utils;

use clap::{CommandFactory, Parser};
use cli::Cli;
use db::services::query_data;
use registry::ProviderRegistry;
use schema::{NGLDataKind, NGLRequest};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    if cli.search_term.is_none() {
        Cli::command().print_help()?;
        return Ok(());
    }

    let database_url = cli
        .database_url
        .clone()
        .unwrap_or_else(|| "sqlite://ngl.db?mode=rwc".to_string());

    let db = db::establish_connection(&database_url).await?;

    let mut request: NGLRequest = cli.into();
    if request.kinds.is_none() {
        request.kinds = Some(vec![
            NGLDataKind::Function,
            NGLDataKind::Package,
            NGLDataKind::Option,
            NGLDataKind::Example,
            NGLDataKind::Guide,
            NGLDataKind::Type,
        ]);
    }

    ProviderRegistry::sync(&db, request.clone()).await?;

    if let Some(ref _term) = request.search_term {
        let response = query_data(&db, &request).await?;
        println!("{}", serde_json::to_string_pretty(&response)?);
    }

    Ok(())
}
