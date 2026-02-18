#![allow(unused)]
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "option_examples")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub option_id: i32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub example_id: i32,
    pub placeholder_key: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::option::Entity",
        from = "Column::OptionId",
        to = "super::option::Column::Id"
    )]
    Option,
    #[sea_orm(
        belongs_to = "super::example::Entity",
        from = "Column::ExampleId",
        to = "super::example::Column::Id"
    )]
    Example,
}

impl Related<super::option::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Option.def()
    }
}

impl Related<super::example::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Example.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

pub struct OptionToExamples;

impl Linked for OptionToExamples {
    type FromEntity = super::option::Entity;
    type ToEntity = super::example::Entity;

    fn link(&self) -> Vec<RelationDef> {
        vec![Relation::Option.def().rev(), Relation::Example.def()]
    }
}

pub struct ExampleToOptions;

impl Linked for ExampleToOptions {
    type FromEntity = super::example::Entity;
    type ToEntity = super::option::Entity;

    fn link(&self) -> Vec<RelationDef> {
        vec![Relation::Example.def().rev(), Relation::Option.def()]
    }
}
