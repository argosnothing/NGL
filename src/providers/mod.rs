use crate::{
    db::{
        entities::{
            example, function, guide, option, package, provider, provider_kind_cache, r#type,
        },
        services::insert,
    },
    schema::{NGLDataKind, NGLRequest},
};
/// If you are writing a provider, read this module carefully.
use async_trait::async_trait;
use chrono::Utc;
use sea_orm::{ActiveValue::Set, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter};

pub mod hjem_docs;
pub mod nixpkgs;
pub mod noogle;

/// Provider Information.
/// Pay special attention to kinds field
/// as this will be used
pub struct ProviderInformation {
    /// What kinds should the provider support.
    /// For every kind in this list, your provider
    /// If you omit a kind, and a request comes in
    /// that does not advertise it supports that kind,
    /// even if it returns data of that kind, it will
    /// not be ran.
    pub kinds: Vec<NGLDataKind>,
    /// Name of the provider
    pub name: String,
    /// We should enforce that a provider credit the source with at least the url.
    pub source: String,
}

/// Every Provider must implement these traits.
/// Providers must do several things:
/// 1. Advertise the [`NGLDataKind`]'s they support through [`kinds`] From
///    ProviderInformation. This is used by the registry to decide when the provider
///    syncs given different requests.
/// 2. For every [`kind`] the provider advertises in it's information, implement the corresponding
///    fetch function against that [`kind`]. If a provider does not support this kind of data, return
///    an empty vec.
/// 3. Document the source of the data.
#[async_trait]
pub trait Provider {
    fn get_info(&self) -> ProviderInformation;

    /// Fetch functions from this provider. Return empty vec if not supported.
    async fn fetch_functions(&mut self) -> Vec<function::ActiveModel>;

    /// Fetch examples from this provider. Return empty vec if not supported.
    async fn fetch_examples(&mut self) -> Vec<example::ActiveModel>;

    /// Fetch guides from this provider. Return empty vec if not supported.
    async fn fetch_guides(&mut self) -> Vec<guide::ActiveModel>;

    /// Fetch options from this provider. Return empty vec if not supported.
    async fn fetch_options(&mut self) -> Vec<option::ActiveModel>;

    /// Fetch packages from this provider. Return empty vec if not supported.
    async fn fetch_packages(&mut self) -> Vec<package::ActiveModel>;

    /// Fetch types from this provider. Return empty vec if not supported.
    async fn fetch_types(&mut self) -> Vec<r#type::ActiveModel>;

    /// This is where the provider inserts their data into the database using the sources.
    /// A provider is responsible for:
    /// 1. Implementing the fetch_* methods for each kind they support
    /// 2. Declaring each supported `kind` in `ProviderInformation.kinds`
    /// 3. This default implementation will call the appropriate fetch_* method and insert
    async fn fetch_and_insert(
        &mut self,
        db: &DatabaseConnection,
        request: NGLRequest,
    ) -> Result<(), DbErr> {
        let kinds = request
            .kinds
            .as_ref()
            .ok_or_else(|| DbErr::Custom("No kinds specified in request".to_string()))?;

        for kind in kinds {
            match kind {
                NGLDataKind::Function => {
                    let models = self.fetch_functions().await;
                    if !models.is_empty() {
                        insert(db, models).await?;
                    }
                }
                NGLDataKind::Example => {
                    let models = self.fetch_examples().await;
                    if !models.is_empty() {
                        insert(db, models).await?;
                    }
                }
                NGLDataKind::Guide => {
                    let models = self.fetch_guides().await;
                    if !models.is_empty() {
                        insert(db, models).await?;
                    }
                }
                NGLDataKind::Option => {
                    let models = self.fetch_options().await;
                    if !models.is_empty() {
                        insert(db, models).await?;
                    }
                }
                NGLDataKind::Package => {
                    let models = self.fetch_packages().await;
                    if !models.is_empty() {
                        insert(db, models).await?;
                    }
                }
                NGLDataKind::Type => {
                    let models = self.fetch_types().await;
                    if !models.is_empty() {
                        insert(db, models).await?;
                    }
                }
            }
        }

        Ok(())
    }

    /// sync handles the logic around when a provider will and wont request from its source.
    /// Providers job is only to insert into the db when asked it to, which is when
    /// the fetch_and_insert function will be ran.
    async fn sync(&mut self, db: &DatabaseConnection, request: NGLRequest) -> Result<(), DbErr> {
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
                age >= chrono::Duration::hours(24)
            } else {
                true
            };

            if needs_sync {
                kinds_to_sync.push(kind.clone());
            }
        }

        if kinds_to_sync.is_empty() {
            println!("All requested kinds cached for {}", &info.name);
            return Ok(());
        }

        let provider_model = provider::ActiveModel {
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

        let sync_request = NGLRequest {
            kinds: Some(kinds_to_sync.clone()),
            ..request
        };

        self.fetch_and_insert(db, sync_request).await?;

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

        Ok(())
    }
}
