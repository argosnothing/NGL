#![allow(unused)]
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "function_examples")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub function_id: i32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub example_id: i32,
    pub placeholder_key: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::function::Entity",
        from = "Column::FunctionId",
        to = "super::function::Column::Id"
    )]
    Function,
    #[sea_orm(
        belongs_to = "super::example::Entity",
        from = "Column::ExampleId",
        to = "super::example::Column::Id"
    )]
    Example,
}

impl Related<super::function::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Function.def()
    }
}

impl Related<super::example::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Example.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

pub struct FunctionToExamples;

impl Linked for FunctionToExamples {
    type FromEntity = super::function::Entity;
    type ToEntity = super::example::Entity;

    fn link(&self) -> Vec<RelationDef> {
        vec![Relation::Function.def().rev(), Relation::Example.def()]
    }
}

pub struct ExampleToFunctions;

impl Linked for ExampleToFunctions {
    type FromEntity = super::example::Entity;
    type ToEntity = super::function::Entity;

    fn link(&self) -> Vec<RelationDef> {
        vec![Relation::Example.def().rev(), Relation::Function.def()]
    }
}
