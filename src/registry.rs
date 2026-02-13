use crate::{
    providers::{Provider, noogle::Noogle, nixpkgs::NixPkgs},
    schema::NGLRequest,
};
use futures::future::join_all;
use sea_orm::{DatabaseConnection, DbErr};

pub struct ProviderRegistry;

impl ProviderRegistry {
    /// Sync all registered providers with the database.
    /// If no kinds are specified in the request, all providers are synced.
    /// Otherwise, only providers that support the requested kinds are synced.
    pub async fn sync(db: &DatabaseConnection, request: NGLRequest) -> Result<(), DbErr> {
        let providers: Vec<Box<dyn Provider + Send>> = vec![
            Box::new(Noogle::new()),
            Box::new(NixPkgs::new()),
        ];

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
