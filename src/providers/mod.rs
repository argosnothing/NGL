use crate::{db::entities::{provider, provider_kind_cache}, schema::NGLRequest};
use chrono::Utc;
use sea_orm::{ActiveValue::Set, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter};

pub mod noogle;
pub mod traits;

pub trait Provider {
    async fn fetch_and_insert(db: &DatabaseConnection, request: NGLRequest) -> Result<(), DbErr>;
    fn get_name() -> String;

    async fn sync(db: &DatabaseConnection, request: NGLRequest) -> Result<(), DbErr> {
        let requested_kinds = request.kinds.as_ref().ok_or_else(|| {
            DbErr::Custom("No kinds specified in request".to_string())
        })?;

        let mut kinds_to_sync = Vec::new();

        for kind in requested_kinds {
            let cache_entry = provider_kind_cache::Entity::find()
                .filter(provider_kind_cache::Column::ProviderName.eq(Self::get_name()))
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
            println!("All requested kinds cached for {}", Self::get_name());
            return Ok(());
        }

        let provider_model = provider::ActiveModel {
            name: Set(Self::get_name()),
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

        Self::fetch_and_insert(db, sync_request).await?;

        for kind in kinds_to_sync {
            let cache_model = provider_kind_cache::ActiveModel {
                provider_name: Set(Self::get_name()),
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
