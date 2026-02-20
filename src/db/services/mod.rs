#![allow(unused)]
use sea_orm::{
    ColumnTrait, ConnectionTrait, DatabaseConnection, DbErr, EntityTrait, FromQueryResult,
    QueryFilter, Statement,
};

use crate::{
    db::entities::{NGLDataEntity, example, function, guide, guide_xref, option, package, r#type},
    schema::{
        ExampleData, FunctionData, GuideData, GuideRef, NGLData, NGLDataKind, NGLDataVariant,
        NGLRaw, NGLRequest, NGLResponse, OptionData, PackageData, TypeData,
    },
};

#[derive(FromQueryResult)]
struct SearchResult {
    entity_id: i32,
    kind: String,
    provider_name: String,
}

pub async fn insert<T>(db: &DatabaseConnection, models: Vec<T>) -> Result<(), DbErr>
where
    T: NGLDataEntity,
{
    let mut remaining = models;
    const CHUNK_SIZE: usize = 150;
    while !remaining.is_empty() {
        let split_at = if remaining.len() > CHUNK_SIZE {
            remaining.len() - CHUNK_SIZE
        } else {
            0
        };
        let chunk = remaining.split_off(split_at);
        T::Entity::insert_many(chunk).exec(db).await?;
    }

    Ok(())
}

#[allow(dead_code)]
pub async fn populate_fts5(db: &DatabaseConnection) -> Result<(), DbErr> {
    db.execute(Statement::from_string(
        db.get_database_backend(),
        "DELETE FROM ngl_search".to_owned(),
    ))
    .await?;

    db.execute(Statement::from_string(
        db.get_database_backend(),
        "INSERT INTO ngl_search (entity_id, kind, provider_name, title, content)
         SELECT id, 'Function', provider_name, name, '' FROM functions"
            .to_owned(),
    ))
    .await?;

    db.execute(Statement::from_string(
        db.get_database_backend(),
        "INSERT INTO ngl_search (entity_id, kind, provider_name, title, content)
         SELECT id, 'Example', provider_name, '', data FROM examples"
            .to_owned(),
    ))
    .await?;

    db.execute(Statement::from_string(
        db.get_database_backend(),
        "INSERT INTO ngl_search (entity_id, kind, provider_name, title, content)
         SELECT id, 'Guide', provider_name, title, '' FROM guides"
            .to_owned(),
    ))
    .await?;

    db.execute(Statement::from_string(
        db.get_database_backend(),
        "INSERT INTO ngl_search (entity_id, kind, provider_name, title, content)
         SELECT id, 'Option', provider_name, name, '' FROM options"
            .to_owned(),
    ))
    .await?;

    db.execute(Statement::from_string(
        db.get_database_backend(),
        "INSERT INTO ngl_search (entity_id, kind, provider_name, title, content)
         SELECT id, 'Package', provider_name, name, name FROM packages"
            .to_owned(),
    ))
    .await?;

    db.execute(Statement::from_string(
        db.get_database_backend(),
        "INSERT INTO ngl_search (entity_id, kind, provider_name, title, content)
         SELECT id, 'Type', provider_name, name, data FROM types"
            .to_owned(),
    ))
    .await?;

    Ok(())
}

pub async fn query_data(
    db: &DatabaseConnection,
    request: &NGLRequest,
) -> Result<Vec<NGLResponse>, DbErr> {
    let search_term = request
        .search_term
        .as_ref()
        .map(|s| format!("\"{}\"*", s.replace("\"", "\"\"")))
        .unwrap_or_else(|| "*".to_string());

    let requested_kinds = request.kinds.as_ref();

    // If examples are requested along with any "host" kind that can contain examples,
    // we'll stitch examples into those hosts and skip standalone example results.
    let host_kinds = [
        NGLDataKind::Guide,
        NGLDataKind::Function,
        NGLDataKind::Option,
        NGLDataKind::Package,
        NGLDataKind::Type,
    ];

    let examples_stitched = requested_kinds
        .map(|k| {
            k.contains(&NGLDataKind::Example) && k.iter().any(|kind| host_kinds.contains(kind))
        })
        .unwrap_or(true);

    let include_examples = requested_kinds
        .map(|k| k.contains(&NGLDataKind::Example))
        .unwrap_or(true);

    let mut query = format!(
        "SELECT entity_id, kind, provider_name FROM ngl_search WHERE ngl_search MATCH '{}'",
        search_term
    );

    if let Some(kinds) = requested_kinds {
        let mut filtered_kinds: Vec<_> = kinds.iter().collect();
        if examples_stitched {
            filtered_kinds.retain(|k| **k != NGLDataKind::Example);
        }
        let kind_filter: Vec<String> = filtered_kinds
            .iter()
            .map(|k| format!("'{:?}'", k))
            .collect();
        if !kind_filter.is_empty() {
            query.push_str(&format!(" AND kind IN ({})", kind_filter.join(",")));
        }
    } else {
        query.push_str(" AND kind != 'Example'");
    }

    if let Some(providers) = &request.providers {
        let provider_filter: Vec<String> = providers.iter().map(|p| format!("'{}'", p)).collect();
        query.push_str(&format!(
            " AND provider_name IN ({})",
            provider_filter.join(",")
        ));
    }

    query.push_str(" ORDER BY rank");

    let search_results: Vec<SearchResult> =
        SearchResult::find_by_statement(Statement::from_string(db.get_database_backend(), query))
            .all(db)
            .await?;

    let mut provider_data: std::collections::HashMap<String, Vec<NGLData>> =
        std::collections::HashMap::new();

    for result in search_results {
        let ngl_data = match result.kind.as_str() {
            "Function" => fetch_function(db, result.entity_id, include_examples).await?,
            "Example" => fetch_example(db, result.entity_id).await?,
            "Guide" => fetch_guide(db, result.entity_id, include_examples).await?,
            "Option" => fetch_option(db, result.entity_id, include_examples).await?,
            "Package" => fetch_package(db, result.entity_id, include_examples).await?,
            "Type" => fetch_type(db, result.entity_id, include_examples).await?,
            _ => continue,
        };

        provider_data
            .entry(result.provider_name)
            .or_insert_with(Vec::new)
            .push(ngl_data);
    }

    let responses: Vec<NGLResponse> = provider_data
        .into_iter()
        .map(|(provider_name, matches)| NGLResponse {
            provider_name,
            matches,
        })
        .collect();

    Ok(responses)
}

async fn fetch_function(
    db: &DatabaseConnection,
    id: i32,
    include_examples: bool,
) -> Result<NGLData, DbErr> {
    let model = function::Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| DbErr::RecordNotFound(format!("Function {}", id)))?;

    let content = match model.format {
        crate::db::enums::documentation_format::DocumentationFormat::Markdown => {
            NGLRaw::Markdown(model.data)
        }
        crate::db::enums::documentation_format::DocumentationFormat::HTML => {
            NGLRaw::HTML(model.data)
        }
        crate::db::enums::documentation_format::DocumentationFormat::PlainText => {
            NGLRaw::PlainText(model.data)
        }
    };

    Ok(NGLData {
        data: NGLDataVariant::Function(FunctionData {
            name: model.name,
            signature: model.signature,
            content,
            source_url: model.source_url,
            source_code_url: model.source_code_url,
            aliases: model.aliases.and_then(|s| serde_json::from_str(&s).ok()),
        }),
    })
}

async fn fetch_example(db: &DatabaseConnection, id: i32) -> Result<NGLData, DbErr> {
    let model = example::Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| DbErr::RecordNotFound(format!("Example {}", id)))?;

    Ok(NGLData {
        data: NGLDataVariant::Example(ExampleData {
            code: model.data,
            language: model.language.map(|lang| lang.to_string()),
            source_link: model.source_link,
            source_kind: model.source_kind,
        }),
    })
}

async fn fetch_guide(
    db: &DatabaseConnection,
    id: i32,
    include_examples: bool,
) -> Result<NGLData, DbErr> {
    let model = guide::Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| DbErr::RecordNotFound(format!("Guide {}", id)))?;

    let parent_info = guide_xref::Entity::find()
        .filter(guide_xref::Column::SubGuideId.eq(id))
        .find_also_related(guide::Entity)
        .one(db)
        .await?
        .and_then(|(_, parent)| parent);

    let parent_guide = parent_info.map(|p| GuideRef {
        id: p.id,
        link: Some(p.link),
        title: Some(p.title),
    });

    let child_xrefs = guide_xref::Entity::find()
        .filter(guide_xref::Column::GuideId.eq(id))
        .all(db)
        .await?;

    let mut sub_guides = Vec::new();
    for xref in child_xrefs {
        if let Some(child) = guide::Entity::find_by_id(xref.sub_guide_id).one(db).await? {
            sub_guides.push(GuideRef {
                id: child.id,
                link: Some(child.link),
                title: Some(child.title),
            });
        }
    }

    let content = match model.format {
        crate::db::enums::documentation_format::DocumentationFormat::Markdown => {
            NGLRaw::Markdown(model.data)
        }
        crate::db::enums::documentation_format::DocumentationFormat::HTML => {
            NGLRaw::HTML(model.data)
        }
        crate::db::enums::documentation_format::DocumentationFormat::PlainText => {
            NGLRaw::PlainText(model.data)
        }
    };

    Ok(NGLData {
        data: NGLDataVariant::Guide(GuideData {
            parent_guide,
            sub_guides,
            link: model.link,
            title: NGLRaw::PlainText(model.title),
            content,
        }),
    })
}

async fn fetch_option(
    db: &DatabaseConnection,
    id: i32,
    include_examples: bool,
) -> Result<NGLData, DbErr> {
    let model = option::Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| DbErr::RecordNotFound(format!("Option {}", id)))?;

    Ok(NGLData {
        data: NGLDataVariant::Option(OptionData {
            name: model.name,
            option_type: model.type_signature,
            default_value: model.default_value,
            description: Some(model.data),
            example: None,
        }),
    })
}

async fn fetch_package(
    db: &DatabaseConnection,
    id: i32,
    include_examples: bool,
) -> Result<NGLData, DbErr> {
    let model = package::Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| DbErr::RecordNotFound(format!("Package {}", id)))?;

    Ok(NGLData {
        data: NGLDataVariant::Package(PackageData {
            name: model.name,
            version: model.version,
            description: model.description,
            homepage: model.homepage,
            license: model.license,
            source_code_url: model.source_code_url,
            broken: model.broken,
            unfree: model.unfree,
        }),
    })
}

async fn fetch_type(
    db: &DatabaseConnection,
    id: i32,
    include_examples: bool,
) -> Result<NGLData, DbErr> {
    let model = r#type::Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| DbErr::RecordNotFound(format!("Type {}", id)))?;

    Ok(NGLData {
        data: NGLDataVariant::Type(TypeData {
            name: model.name,
            description: Some(model.data),
        }),
    })
}
