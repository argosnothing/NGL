use sea_orm::{ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter};

use crate::{
    db::{entities::function, example},
    providers::traits::{ProvidesExamples, ProvidesFunctions},
    schema::{ExampleData, NGLData, NGLDataKind, NGLDataVariant, NGLRequest},
};

pub async fn insert_functions<P: ProvidesFunctions>(
    db: &DatabaseConnection,
    provider: &P,
) -> Result<(), DbErr> {
    let models = provider.get_functions();
    function::Entity::insert_many(models).exec(db).await?;
    Ok(())
}

pub async fn insert_examples<P: ProvidesExamples>(
    db: &DatabaseConnection,
    provider: &P,
) -> Result<(), DbErr> {
    let models = provider.get_examples();
    example::Entity::insert_many(models).exec(db).await?;
    Ok(())
}
pub async fn query_data(
    db: &DatabaseConnection,
    request: &NGLRequest,
) -> Result<Vec<NGLData>, DbErr> {
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

    Ok(results)
}
async fn query_functions_table(
    db: &DatabaseConnection,
    request: &NGLRequest,
) -> Result<Vec<NGLData>, DbErr> {
    let mut query = function::Entity::find();

    if let Some(term) = &request.search_term {}
    if let Some(providers) = &request.providers {
        query = query.filter(function::Column::ProviderName.is_in(providers));
    }

    let models = query.all(db).await?;

    let results: Vec<NGLData> = models
        .into_iter()
        .map(|m| NGLData {
            data: NGLDataVariant::Function(serde_json::from_str(&m.data).unwrap()),
        })
        .collect();

    Ok(results)
}

async fn query_examples_table(
    db: &DatabaseConnection,
    request: &NGLRequest,
) -> Result<Vec<NGLData>, DbErr> {
    let mut query = example::Entity::find();

    if let Some(term) = &request.search_term {
        query = query.filter(example::Column::Data.contains(term))
    }
    if let Some(providers) = &request.providers {
        query = query.filter(example::Column::ProviderName.is_in(providers));
    }

    let models = query.all(db).await?;

    let results: Vec<NGLData> = models
        .into_iter()
        .map(|m| NGLData {
            data: NGLDataVariant::Example(ExampleData {
                code: m.data,
                language: m.language.map(|lang| lang.to_string()),
            }),
        })
        .collect();

    Ok(results)
}
