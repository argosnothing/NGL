mod db;
mod providers;
mod registry;
mod schema;

use db::services::query_data;
use registry::ProviderRegistry;
use schema::{NGLDataKind, NGLDataVariant, NGLRequest};

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

    let request = NGLRequest {
        search_term: Some(term.clone()),
        providers: None,
        kinds: Some(vec![NGLDataKind::Function, NGLDataKind::Example]),
    };

    ProviderRegistry::sync(&db, request.clone()).await?;

    // Example NGL response
    let response = query_data(&db, &request).await?;

    for provider_response in response {
        for data in provider_response.matches {
            match &data.data {
                NGLDataVariant::Function(f) => println!("{}", f.name),
                NGLDataVariant::Example(_) => {}
                NGLDataVariant::Guide(g) => match &g.title {
                    schema::NGLRaw::Markdown(s)
                    | schema::NGLRaw::HTML(s)
                    | schema::NGLRaw::PlainText(s) => println!("{}", s),
                },
                NGLDataVariant::Option(o) => println!("{}", o.name),
                NGLDataVariant::Package(p) => println!("{}", p.name),
                NGLDataVariant::Type(t) => println!("{}", t.name),
            }
        }
    }

    Ok(())
}
