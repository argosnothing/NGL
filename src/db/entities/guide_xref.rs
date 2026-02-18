#![allow(unused)]
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

impl Related<super::guide::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ParentGuide.def()
    }
}

pub struct GuideToSubGuides;

impl Linked for GuideToSubGuides {
    type FromEntity = super::guide::Entity;
    type ToEntity = super::guide::Entity;

    fn link(&self) -> Vec<RelationDef> {
        vec![Relation::ParentGuide.def().rev(), Relation::SubGuide.def()]
    }
}

pub struct SubGuideToParents;

impl Linked for SubGuideToParents {
    type FromEntity = super::guide::Entity;
    type ToEntity = super::guide::Entity;

    fn link(&self) -> Vec<RelationDef> {
        vec![Relation::SubGuide.def().rev(), Relation::ParentGuide.def()]
    }
}
