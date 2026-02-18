use chrono::Utc;
use sea_orm::{ActiveValue::Set, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter};

use crate::{
    NGLDataKind,
    db::entities::{
        example, function, guide, guide_example, guide_xref, option, package, provider_kind_cache,
        r#type,
    },
};

pub const HOST_KINDS: [NGLDataKind; 5] = [
    NGLDataKind::Guide,
    NGLDataKind::Function,
    NGLDataKind::Option,
    NGLDataKind::Package,
    NGLDataKind::Type,
];

pub async fn determine_kinds_to_sync(
    db: &DatabaseConnection,
    requested_kinds: &[NGLDataKind],
    supported_kinds: &[NGLDataKind],
    provider_name: &str,
    sync_interval_hours: i64,
) -> Result<Vec<NGLDataKind>, DbErr> {
    let mut kinds_to_sync = Vec::new();

    for kind in requested_kinds {
        if !supported_kinds.contains(kind) {
            continue;
        }

        let cache_entry = provider_kind_cache::Entity::find()
            .filter(provider_kind_cache::Column::ProviderName.eq(provider_name))
            .filter(provider_kind_cache::Column::Kind.eq(format!("{:?}", kind)))
            .one(db)
            .await?;

        let needs_sync = if let Some(entry) = cache_entry {
            let age = Utc::now().signed_duration_since(entry.last_synced);
            age >= chrono::Duration::hours(sync_interval_hours)
        } else {
            true
        };

        if needs_sync {
            kinds_to_sync.push(kind.clone());
        }
    }

    let syncing_host = kinds_to_sync.iter().any(|k| HOST_KINDS.contains(k));
    if syncing_host
        && supported_kinds.contains(&NGLDataKind::Example)
        && !kinds_to_sync.contains(&NGLDataKind::Example)
    {
        let example_cache = provider_kind_cache::Entity::find()
            .filter(provider_kind_cache::Column::ProviderName.eq(provider_name))
            .filter(provider_kind_cache::Column::Kind.eq(format!("{:?}", NGLDataKind::Example)))
            .one(db)
            .await?;

        if example_cache.is_some() {
            kinds_to_sync.push(NGLDataKind::Example);
        }
    }

    // the opposite of the last thingy
    if kinds_to_sync.contains(&NGLDataKind::Example) {
        for host_kind in &HOST_KINDS {
            if supported_kinds.contains(host_kind) && !kinds_to_sync.contains(host_kind) {
                let host_cache = provider_kind_cache::Entity::find()
                    .filter(provider_kind_cache::Column::ProviderName.eq(provider_name))
                    .filter(provider_kind_cache::Column::Kind.eq(format!("{:?}", host_kind)))
                    .one(db)
                    .await?;

                if host_cache.is_some() {
                    kinds_to_sync.push(host_kind.clone());
                }
            }
        }
    }

    Ok(kinds_to_sync)
}

// TODO: We should be able to use cascades for this right??? do those exist in lite??
pub async fn delete_provider_kind_data(
    db: &DatabaseConnection,
    kind: &NGLDataKind,
    provider_name: &str,
) -> Result<(), DbErr> {
    match kind {
        NGLDataKind::Function => {
            function::Entity::delete_many()
                .filter(function::Column::ProviderName.eq(provider_name))
                .exec(db)
                .await?;
        }
        NGLDataKind::Example => {
            let guide_ids: Vec<i32> = guide::Entity::find()
                .filter(guide::Column::ProviderName.eq(provider_name))
                .all(db)
                .await?
                .into_iter()
                .map(|g| g.id)
                .collect();
            if !guide_ids.is_empty() {
                guide_example::Entity::delete_many()
                    .filter(guide_example::Column::GuideId.is_in(guide_ids))
                    .exec(db)
                    .await?;
            }
            example::Entity::delete_many()
                .filter(example::Column::ProviderName.eq(provider_name))
                .exec(db)
                .await?;
        }
        NGLDataKind::Guide => {
            let guide_ids: Vec<i32> = guide::Entity::find()
                .filter(guide::Column::ProviderName.eq(provider_name))
                .all(db)
                .await?
                .into_iter()
                .map(|g| g.id)
                .collect();
            if !guide_ids.is_empty() {
                guide_xref::Entity::delete_many()
                    .filter(guide_xref::Column::GuideId.is_in(guide_ids.clone()))
                    .exec(db)
                    .await?;
                guide_xref::Entity::delete_many()
                    .filter(guide_xref::Column::SubGuideId.is_in(guide_ids.clone()))
                    .exec(db)
                    .await?;
                guide_example::Entity::delete_many()
                    .filter(guide_example::Column::GuideId.is_in(guide_ids))
                    .exec(db)
                    .await?;
            }
            guide::Entity::delete_many()
                .filter(guide::Column::ProviderName.eq(provider_name))
                .exec(db)
                .await?;
        }
        NGLDataKind::Option => {
            option::Entity::delete_many()
                .filter(option::Column::ProviderName.eq(provider_name))
                .exec(db)
                .await?;
        }
        NGLDataKind::Package => {
            package::Entity::delete_many()
                .filter(package::Column::ProviderName.eq(provider_name))
                .exec(db)
                .await?;
        }
        NGLDataKind::Type => {
            r#type::Entity::delete_many()
                .filter(r#type::Column::ProviderName.eq(provider_name))
                .exec(db)
                .await?;
        }
    }
    Ok(())
}

pub async fn update_kind_cache(
    db: &DatabaseConnection,
    kinds: &[NGLDataKind],
    provider_name: &str,
) -> Result<(), DbErr> {
    for kind in kinds {
        let cache_model = provider_kind_cache::ActiveModel {
            provider_name: Set(provider_name.to_string()),
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
