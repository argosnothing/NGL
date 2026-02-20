use sea_orm::entity::prelude::*;

use crate::{
    NGLDataKind,
    db::{entities::NGLDataEntity, enums::language::Language},
};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "examples")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub provider_name: String,

    pub language: Option<Language>,
    pub data: String,
    pub source_kind: Option<NGLDataKind>,
    pub source_link: Option<String>,
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
    const KIND: NGLDataKind = NGLDataKind::Example;
}
