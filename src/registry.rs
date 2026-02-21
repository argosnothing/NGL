#[cfg(feature = "example")]
use crate::providers::example::Example;
#[cfg(feature = "nixos_manual")]
use crate::providers::nixos_manual::NixosManual;
#[cfg(feature = "nixpkgs")]
use crate::providers::nixpkgs::NixPkgs;
#[cfg(feature = "noogle")]
use crate::providers::noogle::Noogle;
use crate::{
    cli::progress::run_progress_ui,
    providers::{Provider, meta::MetaProvider},
    schema::NGLRequest,
};
use futures::future::join_all;
use sea_orm::{DatabaseConnection, DbErr};
use std::path::PathBuf;
use tokio::sync::broadcast;

pub struct ProviderRegistry;

impl ProviderRegistry {
    /// Sync all registered providers with the database.
    /// If no kinds are specified in the request, all providers are synced.
    /// Otherwise, only providers that support the requested kinds are synced.
    /// Automatically loads templates.json from current directory if it exists.
    pub async fn sync(db: &DatabaseConnection, request: NGLRequest) -> Result<(), DbErr> {
        let config_path = PathBuf::from("templates.json");
        let config = if config_path.exists() {
            Some(config_path)
        } else {
            None
        };
        Self::sync_with_config(db, request, config).await
    }

    /// Sync with optional meta provider config file
    pub async fn sync_with_config(
        db: &DatabaseConnection,
        request: NGLRequest,
        config_path: Option<PathBuf>,
    ) -> Result<(), DbErr> {
        #[allow(unused_mut)]
        let mut providers: Vec<Box<dyn Provider + Send>> = vec![];

        #[cfg(feature = "example")]
        providers.push(Box::new(Example::new()));
        // ADD YOUR PROVIDERS HERE, IDEALLY ALSO TIE THEM INTO A FEATURE
        // SO OTHER PROGRAMS CAN CHOOSE TO COMPILE THEM OUT
        #[cfg(feature = "noogle")]
        providers.push(Box::new(Noogle::new()));
        #[cfg(feature = "nixpkgs")]
        providers.push(Box::new(NixPkgs::new()));
        #[cfg(feature = "nixos_manual")]
        providers.push(Box::new(NixosManual::new()));

        if let Some(path) = config_path {
            match MetaProvider::from_file(&path) {
                Ok(meta) => {
                    let meta_providers = meta.build_providers();
                    providers.extend(meta_providers);
                }
                Err(e) => {
                    eprintln!("Warning: failed to load meta provider config: {}", e);
                }
            }
        }

        // broadcaster to send status events to, right now we just have a simple progress tui in term, but
        // this will work later on when we need to provide status to calling code.
        let (status_tx, _) = broadcast::channel(1024);
        tokio::spawn(run_progress_ui(status_tx.subscribe()));
        let sync_futures: Vec<_> = providers
            .into_iter()
            .filter_map(|mut provider| {
                if provider.get_info().kinds.iter().any(|provider_kind| {
                    request
                        .kinds
                        .as_ref()
                        .map_or(false, |kinds| kinds.contains(provider_kind))
                }) {
                    let provider_name = provider.get_info().name.clone();
                    Some({
                        let request_clone = request.clone();
                        let db_clone = db.clone();
                        let status_clone = status_tx.clone();
                        async move {
                            let result = provider
                                .refresh(&db_clone, request_clone, status_clone)
                                .await;
                            (provider_name, result)
                        }
                    })
                } else {
                    None
                }
            })
            .collect();

        let results = join_all(sync_futures).await;

        let mut reindex = false;
        let mut errors = Vec::new();
        for (provider_name, result) in results {
            match result {
                Ok(synced) => {
                    if synced {
                        reindex = true;
                    }
                }
                Err(e) => {
                    errors.push(format!("{}: {}", provider_name, e));
                }
            }
        }

        if reindex {
            eprint!("Reindexing FTS5 tables...");
            crate::db::services::populate_fts5(db).await?;
        }

        if !errors.is_empty() {
            eprintln!("\nSync errors:");
            for err in &errors {
                eprintln!("  {}", err);
            }
        }

        Ok(())
    }
}
