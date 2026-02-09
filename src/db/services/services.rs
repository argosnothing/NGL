use sea_orm::{ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter};

use crate::{
    db::entities::{example, function},
    schema::{ExampleData, FunctionData, NGLData, NGLDataKind, NGLDataVariant, NGLRaw, NGLRequest, NGLResponse},
};

pub async fn insert_functions(
    db: &DatabaseConnection,
    models: Vec<function::ActiveModel>,
) -> Result<(), DbErr> {
    function::Entity::insert_many(models).exec(db).await?;
    Ok(())
}

pub async fn insert_examples(
    db: &DatabaseConnection,
    models: Vec<example::ActiveModel>,
) -> Result<(), DbErr> {
    example::Entity::insert_many(models).exec(db).await?;
    Ok(())
}
pub async fn query_data(
    db: &DatabaseConnection,
    request: &NGLRequest,
) -> Result<Vec<NGLResponse>, DbErr> {
    let mut results = Vec::new();

    let kinds = request
        .kinds
        .as_ref()
        .map(|k| k.clone())
        .unwrap_or_else(|| vec![NGLDataKind::Function, NGLDataKind::Example]);

    for kind in kinds {
        match kind {
            NGLDataKind::Function => {
                let functions = query_functions_table(db, request).await?;
                results.extend(functions);
            }
            NGLDataKind::Example => {
                let examples = query_examples_table(db, request).await?;
                results.extend(examples);
            }
            NGLDataKind::Guide => todo!(),
            NGLDataKind::Option => todo!(),
            NGLDataKind::Package => todo!(),
            NGLDataKind::Type => todo!(),
        }
    }

    let mut provider_map: std::collections::HashMap<String, Vec<NGLData>> = std::collections::HashMap::new();
    
    for (provider_name, data) in results {
        provider_map.entry(provider_name).or_insert_with(Vec::new).push(data);
    }

    let responses: Vec<NGLResponse> = provider_map
        .into_iter()
        .map(|(provider_name, matches)| NGLResponse {
            provider_name,
            matches,
        })
        .collect();

    Ok(responses)
}
async fn query_functions_table(
    db: &DatabaseConnection,
    request: &NGLRequest,
) -> Result<Vec<(String, NGLData)>, DbErr> {
    let mut query = function::Entity::find();

    if let Some(term) = &request.search_term {
        query = query.filter(function::Column::Name.contains(term));
    }
    if let Some(providers) = &request.providers {
        query = query.filter(function::Column::ProviderName.is_in(providers));
    }

    let models = query.all(db).await?;

    let results: Vec<(String, NGLData)> = models
        .into_iter()
        .map(|m| {
            let provider_name = m.provider_name.clone();
            let content = match m.format {
                crate::db::enums::documentation_format::DocumentationFormat::Markdown => NGLRaw::Markdown(m.data),
                crate::db::enums::documentation_format::DocumentationFormat::HTML => NGLRaw::HTML(m.data),
                crate::db::enums::documentation_format::DocumentationFormat::PlainText => NGLRaw::PlainText(m.data),
            };
            (provider_name, NGLData {
                data: NGLDataVariant::Function(FunctionData {
                    name: m.name,
                    signature: Some(m.signature),
                    content,
                }),
            })
        })
        .collect();

    Ok(results)
}

async fn query_examples_table(
    db: &DatabaseConnection,
    request: &NGLRequest,
) -> Result<Vec<(String, NGLData)>, DbErr> {
    let mut query = example::Entity::find();

    if let Some(term) = &request.search_term {
        query = query.filter(example::Column::Data.contains(term))
    }
    if let Some(providers) = &request.providers {
        query = query.filter(example::Column::ProviderName.is_in(providers));
    }

    let models = query.all(db).await?;

    let results: Vec<(String, NGLData)> = models
        .into_iter()
        .map(|m| {
            let provider_name = m.provider_name.clone();
            (provider_name, NGLData {
                data: NGLDataVariant::Example(ExampleData {
                    code: m.data,
                    language: m.language.map(|lang| lang.to_string()),
                }),
            })
        })
        .collect();

    Ok(results)
}
