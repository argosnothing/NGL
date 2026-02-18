#![allow(unused)]
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "type_examples")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub type_id: i32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub example_id: i32,
    pub placeholder_key: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::r#type::Entity",
        from = "Column::TypeId",
        to = "super::r#type::Column::Id"
    )]
    Type,
    #[sea_orm(
        belongs_to = "super::example::Entity",
        from = "Column::ExampleId",
        to = "super::example::Column::Id"
    )]
    Example,
}

impl Related<super::r#type::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Type.def()
    }
}

impl Related<super::example::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Example.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

pub struct TypeToExamples;

impl Linked for TypeToExamples {
    type FromEntity = super::r#type::Entity;
    type ToEntity = super::example::Entity;

    fn link(&self) -> Vec<RelationDef> {
        vec![Relation::Type.def().rev(), Relation::Example.def()]
    }
}

pub struct ExampleToTypes;

impl Linked for ExampleToTypes {
    type FromEntity = super::example::Entity;
    type ToEntity = super::r#type::Entity;

    fn link(&self) -> Vec<RelationDef> {
        vec![Relation::Example.def().rev(), Relation::Type.def()]
    }
}
