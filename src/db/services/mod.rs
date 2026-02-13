use sea_orm::{
    ConnectionTrait, DatabaseConnection, DbErr, EntityTrait, FromQueryResult, Statement,
};

use crate::{
    db::entities::{NGLDataEntity, example, function, guide, option, package, r#type},
    schema::{
        ExampleData, FunctionData, GuideData, NGLData, NGLDataVariant, NGLRaw, NGLRequest,
        NGLResponse, OptionData, PackageData, TypeData,
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
    // SQLite has a limit on the number of SQL variables per statement (commonly 999).
    // Insert in chunks to avoid exceeding that limit when inserting many rows.
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
         SELECT id, 'Guide', provider_name, title, data FROM guides"
            .to_owned(),
    ))
    .await?;

    db.execute(Statement::from_string(
        db.get_database_backend(),
        "INSERT INTO ngl_search (entity_id, kind, provider_name, title, content)
         SELECT id, 'Option', provider_name, name, data FROM options"
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

    let mut query = format!(
        "SELECT entity_id, kind, provider_name FROM ngl_search WHERE ngl_search MATCH '{}'",
        search_term
    );

    if let Some(kinds) = &request.kinds {
        let kind_filter: Vec<String> = kinds.iter().map(|k| format!("'{:?}'", k)).collect();
        query.push_str(&format!(" AND kind IN ({})", kind_filter.join(",")));
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
            "Function" => fetch_function(db, result.entity_id).await?,
            "Example" => fetch_example(db, result.entity_id).await?,
            "Guide" => fetch_guide(db, result.entity_id).await?,
            "Option" => fetch_option(db, result.entity_id).await?,
            "Package" => fetch_package(db, result.entity_id).await?,
            "Type" => fetch_type(db, result.entity_id).await?,
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

async fn fetch_function(db: &DatabaseConnection, id: i32) -> Result<NGLData, DbErr> {
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
        }),
    })
}

async fn fetch_guide(db: &DatabaseConnection, id: i32) -> Result<NGLData, DbErr> {
    let model = guide::Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| DbErr::RecordNotFound(format!("Guide {}", id)))?;

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
            title: NGLRaw::PlainText(model.title),
            content,
        }),
    })
}

async fn fetch_option(db: &DatabaseConnection, id: i32) -> Result<NGLData, DbErr> {
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

async fn fetch_package(db: &DatabaseConnection, id: i32) -> Result<NGLData, DbErr> {
    let model = package::Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| DbErr::RecordNotFound(format!("Package {}", id)))?;

    Ok(NGLData {
        data: NGLDataVariant::Package(PackageData {
            name: model.name,
            version: model.version,
            description: Some(model.data),
        }),
    })
}

async fn fetch_type(db: &DatabaseConnection, id: i32) -> Result<NGLData, DbErr> {
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
