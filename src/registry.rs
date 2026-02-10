use crate::{
    providers::{Provider, noogle::Noogle},
    schema::NGLRequest,
};
use sea_orm::{DatabaseConnection, DbErr};

macro_rules! register_providers {
    ($($provider:ty),*) => {
        /// Iterate over every provider against the NGL request,
        /// If a provider does not provide data in the currect request, do not
        /// request any syncing from that provider.
        pub async fn sync_all_providers(db: &DatabaseConnection, request: NGLRequest) -> Result<(), DbErr> {
            $(
                {
                    let requested_kinds = request.kinds.as_ref().ok_or_else(|| {
                        DbErr::Custom("No kinds specified in request".to_string())
                    })?;

                    if requested_kinds.iter().any(|k| <$provider>::get_info().kinds.contains(k)) {
                        let mut provider = <$provider>::new();
                        provider.sync(db, request.clone()).await?;
                    }
                }
            )*
            Ok(())
        }
    };
}

register_providers!(Noogle);

pub struct ProviderRegistry;

impl ProviderRegistry {
    /// To add your provider to the registry for syncing to NGL Db, add your provider struct to
    /// register_providers!(x,y,z)
    /// src/registry.rs
    pub async fn sync(db: &DatabaseConnection, request: NGLRequest) -> Result<(), DbErr> {
        sync_all_providers(db, request).await
    }
}
