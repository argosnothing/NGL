use sea_orm::entity::prelude::*;

use crate::{
    NGLDataKind,
    db::{entities::NGLDataEntity, enums::documentation_format::DocumentationFormat},
};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "functions")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    pub name: String,
    pub format: DocumentationFormat,
    pub signature: Option<String>,
    pub provider_name: String,
    pub data: String,
    pub source_url: Option<String>,
    pub source_code_url: Option<String>,
    pub aliases: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::provider::Entity",
        from = "Column::ProviderName",
        to = "super::provider::Column::Name"
    )]
    Provider,
}

impl Related<super::provider::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Provider.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl NGLDataEntity for ActiveModel {
    const KIND: NGLDataKind = NGLDataKind::Function;
}
