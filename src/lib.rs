mod db;
mod providers;
mod registry;
pub mod schema;

pub use schema::{
    ExampleData, FunctionData, GuideData, NGLData, NGLDataKind, NGLDataVariant, NGLRaw, NGLRequest,
    NGLResponse, OptionData, PackageData, TypeData,
};

use registry::ProviderRegistry;
use sea_orm::DbErr;

use crate::db::services::query_data;

pub async fn query(request: NGLRequest) -> Result<Vec<NGLResponse>, DbErr> {
    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://ngl.db?mode=rwc".to_string());

    let db = db::establish_connection(&database_url).await?;

    ProviderRegistry::sync(&db, request.clone()).await?;

    let response = query_data(&db, &request).await?;

    Ok(response)
}
