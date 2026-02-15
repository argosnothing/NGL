use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "guide_xrefs")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub guide_id: i32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub sub_guide_id: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::guide::Entity",
        from = "Column::GuideId",
        to = "super::guide::Column::Id"
    )]
    ParentGuide,
    #[sea_orm(
        belongs_to = "super::guide::Entity",
        from = "Column::SubGuideId",
        to = "super::guide::Column::Id"
    )]
    SubGuide,
}

impl ActiveModelBehavior for ActiveModel {}
