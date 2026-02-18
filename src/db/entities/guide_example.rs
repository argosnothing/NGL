#![allow(unused)]
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "guide_examples")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub guide_id: i32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub example_id: i32,
    pub placeholder_key: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::guide::Entity",
        from = "Column::GuideId",
        to = "super::guide::Column::Id"
    )]
    Guide,
    #[sea_orm(
        belongs_to = "super::example::Entity",
        from = "Column::ExampleId",
        to = "super::example::Column::Id"
    )]
    Example,
}

impl Related<super::guide::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Guide.def()
    }
}

impl Related<super::example::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Example.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

pub struct GuideToExamples;

impl Linked for GuideToExamples {
    type FromEntity = super::guide::Entity;
    type ToEntity = super::example::Entity;

    fn link(&self) -> Vec<RelationDef> {
        vec![Relation::Guide.def().rev(), Relation::Example.def()]
    }
}

pub struct ExampleToGuides;

impl Linked for ExampleToGuides {
    type FromEntity = super::example::Entity;
    type ToEntity = super::guide::Entity;

    fn link(&self) -> Vec<RelationDef> {
        vec![Relation::Example.def().rev(), Relation::Guide.def()]
    }
}
