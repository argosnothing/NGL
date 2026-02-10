use crate::{
    providers::{Provider, noogle::Noogle},
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
        let providers: Vec<Box<dyn Provider + Send>> = vec![Box::new(Noogle::new())];

        let sync_futures: Vec<_> = providers
            .into_iter()
            .filter_map(|mut provider| {
                let should_sync = match &request.kinds {
                    None => true,
                    Some(requested_kinds) => {
                        let info = provider.get_info();
                        requested_kinds.iter().any(|k| info.kinds.contains(k))
                    }
                };

                if should_sync {
                    let request_clone = request.clone();
                    let db_clone = db.clone();
                    Some(async move { provider.sync(&db_clone, request_clone).await })
                } else {
                    None
                }
            })
            .collect();

        let results = join_all(sync_futures).await;

        for result in results {
            result?;
        }

        Ok(())
    }
}
