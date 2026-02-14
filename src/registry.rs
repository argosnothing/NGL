#[cfg(feature = "nixpkgs")]
use crate::providers::nixpkgs::NixPkgs;
#[cfg(feature = "noogle")]
use crate::providers::noogle::Noogle;
use crate::{
    providers::{Provider, meta::MetaProvider},
    schema::NGLRequest,
};
use futures::future::join_all;
use sea_orm::{DatabaseConnection, DbErr};
use std::path::PathBuf;

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

        // ADD YOUR PROVIDERS HERE, IDEALLY ALSO TIE THEM INTO A FEATURE
        // SO OTHER PROGRAMS CAN CHOOSE TO COMPILE THEM OUT
        #[cfg(feature = "noogle")]
        providers.push(Box::new(Noogle::new()));
        #[cfg(feature = "nixpkgs")]
        providers.push(Box::new(NixPkgs::new()));

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

        let sync_futures: Vec<_> = providers
            .into_iter()
            .map(|mut provider| {
                let request_clone = request.clone();
                let db_clone = db.clone();
                async move { provider.refresh(&db_clone, request_clone).await }
            })
            .collect();

        let results = join_all(sync_futures).await;

        let mut reindex = false;
        for result in results {
            let synced = result?;
            if synced {
                reindex = true;
            }
        }

        if reindex {
            eprint!("Reindexing FTS5 tables...");
            crate::db::services::populate_fts5(db).await?;
        }
        Ok(())
    }
}
