use crate::{
    providers::{Provider, noogle::Noogle},
    schema::NGLRequest,
};
use sea_orm::{DatabaseConnection, DbErr};

macro_rules! register_providers {
    ($($provider:ty),*) => {
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
    pub async fn sync(db: &DatabaseConnection, request: NGLRequest) -> Result<(), DbErr> {
        sync_all_providers(db, request).await
    }
}
