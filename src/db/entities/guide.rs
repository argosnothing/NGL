use sea_orm::entity::prelude::*;

use crate::{
    NGLDataKind,
    db::{entities::NGLDataEntity, enums::documentation_format::DocumentationFormat},
};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "guides")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    /// link that takes you to the url + section
    pub link: String,
    pub provider_name: String,

    pub title: String,
    pub format: DocumentationFormat,
    pub data: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::provider::Entity",
        from = "Column::ProviderName",
        to = "super::provider::Column::Name"
    )]
    Provider,
    #[sea_orm(has_many = "super::guide_xref::Entity")]
    GuideXrefsAsParent,
    #[sea_orm(has_many = "super::guide_xref::Entity")]
    GuideXrefsAsSubGuide,
}

impl Related<super::provider::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Provider.def()
    }
}

impl Related<super::guide_xref::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::GuideXrefsAsParent.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl NGLDataEntity for ActiveModel {
    const KIND: NGLDataKind = NGLDataKind::Guide;
}
