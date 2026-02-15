use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use sea_orm::{ActiveValue::Set, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter};

use crate::{NGLDataKind, NGLRequest, db::entities::{provider, provider_kind_cache}, providers::{DbSink, ProviderInformation, Sink}};

#[async_trait]
pub trait Provider: Send {
    /// Returns metadata about this provider.
    /// It's important that each provider has a unique name, as this is used
    /// to create its database table. :)
    /// # Examples:
    /// ```ignore
    /// fn get_info(&self) -> ProviderInformation {
    ///     ProviderInformation {
    ///         name: "my_provider".to_string(),
    ///         source: "https://example.com".to_string(),
    ///         kinds: vec![NGLDataKind::Function, NGLDataKind::Example],
    ///     }
    /// }
    /// ```
    fn get_info(&self) -> ProviderInformation;

    /// The role of the provider is to
    /// 1. Advertise what kind of data it can provide through `get_info()`
    /// 2. Pull raw data from a source, transform it into our internal format, and emit it through the provided `Sink`.
    /// 3. implement this `sync()` method that emits events representing the data it provides.
    /// -  A bit of clarifation, kinds advertised by ProviderInformation->kinds informs the
    ///    registry of what kinds the provider can work with.
    ///    The `kinds` specified in the sync method are the kinds of data that are actually requesting
    ///    to be synced in the current sync.
    ///
    /// # Examples:
    /// ```ignore
    /// async fn sync(&mut self, sink: Arc<dyn Sink>, kinds: &[NGLDataKind]) -> Result<(), DbErr> {
    ///     if kinds.contains(&NGLDataKind::Function) {
    ///         for func in self.fetch_functions().await {
    ///             sink.emit(ProviderEvent::Function(func)).await?;
    ///         }
    ///     }
    ///     Ok(())
    /// }
    /// ```
    async fn sync(&mut self, sink: Arc<dyn Sink>, kinds: &[NGLDataKind]) -> Result<(), DbErr>;

    async fn refresh(
        &mut self,
        db: &DatabaseConnection,
        request: NGLRequest,
    ) -> Result<bool, DbErr> {
        let requested_kinds = request
            .kinds
            .as_ref()
            .ok_or_else(|| DbErr::Custom("No kinds specified in request".to_string()))?;

        let info = self.get_info();
        let supported_kinds = info.kinds;
        let mut kinds_to_sync = Vec::new();

        for kind in requested_kinds {
            if !supported_kinds.contains(kind) {
                continue;
            }

            let cache_entry = provider_kind_cache::Entity::find()
                .filter(provider_kind_cache::Column::ProviderName.eq(&info.name))
                .filter(provider_kind_cache::Column::Kind.eq(format!("{:?}", kind)))
                .one(db)
                .await?;

            let needs_sync = if let Some(entry) = cache_entry {
                let age = Utc::now().signed_duration_since(entry.last_synced);
                age >= chrono::Duration::hours(info.sync_interval_hours.unwrap_or(24) as i64)
            } else {
                true
            };

            if needs_sync {
                println!("{} is Syncing data for {}", &info.name, &kind);
                kinds_to_sync.push(kind.clone());
            }
        }

        if kinds_to_sync.is_empty() {
            println!("All requested kinds cached for {}", &info.name);
            return Ok(false);
        }

        let provider_model = crate::db::entities::provider::ActiveModel {
            name: Set(info.name.clone()),
            last_updated: Set(Utc::now().into()),
        };
        provider::Entity::insert(provider_model)
            .on_conflict(
                sea_orm::sea_query::OnConflict::column(provider::Column::Name)
                    .update_column(provider::Column::LastUpdated)
                    .to_owned(),
            )
            .exec(db)
            .await?;

        let sink = Arc::new(DbSink::new(db.clone()));
        self.sync(sink.clone(), &kinds_to_sync).await?;
        sink.flush().await?;

        for kind in kinds_to_sync {
            let cache_model = provider_kind_cache::ActiveModel {
                provider_name: Set(info.name.clone()),
                kind: Set(format!("{:?}", kind)),
                last_synced: Set(Utc::now().into()),
            };
            provider_kind_cache::Entity::insert(cache_model)
                .on_conflict(
                    sea_orm::sea_query::OnConflict::columns([
                        provider_kind_cache::Column::ProviderName,
                        provider_kind_cache::Column::Kind,
                    ])
                    .update_column(provider_kind_cache::Column::LastSynced)
                    .to_owned(),
                )
                .exec(db)
                .await?;
        }

        Ok(true)
    }

    fn new() -> Self
    where
        Self: Default,
    {
        Self::default()
    }
}
