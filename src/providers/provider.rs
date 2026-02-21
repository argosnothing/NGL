use async_trait::async_trait;
use chrono::Utc;
use sea_orm::{ActiveValue::Set, DatabaseConnection, DbErr, EntityTrait};

use crate::{
    NGLDataKind, NGLRequest,
    db::entities::provider,
    providers::{
        EventChannel, ProviderInformation, channel::StatusEvent, create_event_channel, sync,
    },
};

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
    /// 2. Pull raw data from a source, transform it into our internal format, and emit it through the provided emitter.
    /// 3. implement this `sync()` method that emits events representing the data it provides.
    /// -  A bit of clarifation, kinds advertised by ProviderInformation->kinds informs the
    ///    registry of what kinds the provider can work with.
    ///    The `kinds` specified in the sync method are the kinds of data that are actually requesting
    ///    to be synced in the current sync.
    ///
    /// # Examples:
    /// ```ignore
    /// async fn sync(&mut self, channel: &EventChannel, kinds: &[NGLDataKind]) -> Result<(), DbErr> {
    ///     if kinds.contains(&NGLDataKind::Function) {
    ///         for func in self.fetch_functions().await {
    ///             channel.send(ProviderEvent::Function(func)).await;
    ///         }
    ///     }
    ///     Ok(())
    /// }
    /// ```
    async fn sync(&mut self, channel: &EventChannel, kinds: &[NGLDataKind]) -> Result<(), DbErr>;

    async fn refresh(
        &mut self,
        db: &DatabaseConnection,
        request: NGLRequest,
        status: tokio::sync::broadcast::Sender<StatusEvent>,
    ) -> Result<bool, DbErr> {
        let requested_kinds = request
            .kinds
            .as_ref()
            .ok_or_else(|| DbErr::Custom("No kinds specified in request".to_string()))?;

        let info = self.get_info();
        let kinds_to_sync = sync::determine_kinds_to_sync(
            db,
            requested_kinds,
            &info.kinds,
            &info.name,
            info.sync_interval_hours.unwrap_or(24) as i64,
        )
        .await?;

        if kinds_to_sync.is_empty() && info.kinds.iter().any(|pk| requested_kinds.contains(pk)) {
            return Ok(false);
        }

        // Upsert provider record
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

        for kind in &kinds_to_sync {
            sync::delete_provider_kind_data(db, kind, &info.name).await?;
        }

        let (channel, consumer_handle) =
            create_event_channel(self.get_info().name, db.clone(), status);

        let update_handle = tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_millis(200)).await;
            }
        });

        self.sync(&channel, &kinds_to_sync).await?;
        drop(channel);
        consumer_handle
            .await
            .map_err(|e| DbErr::Custom(format!("Consumer task panicked: {}", e)))??;

        update_handle.abort();

        // Update cache timestamps
        sync::update_kind_cache(db, &kinds_to_sync, &info.name).await?;

        Ok(true)
    }

    fn new() -> Self
    where
        Self: Default,
    {
        Self::default()
    }
}
