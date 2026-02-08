use sea_orm::{DatabaseConnection, DbErr, EntityTrait};

use crate::{
    db::{entities::function, example},
    providers::traits::{ProvidesExamples, ProvidesFunctions},
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
