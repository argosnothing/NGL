#![allow(unused)]
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "package_examples")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub package_id: i32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub example_id: i32,
    pub placeholder_key: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::package::Entity",
        from = "Column::PackageId",
        to = "super::package::Column::Id"
    )]
    Package,
    #[sea_orm(
        belongs_to = "super::example::Entity",
        from = "Column::ExampleId",
        to = "super::example::Column::Id"
    )]
    Example,
}

impl Related<super::package::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Package.def()
    }
}

impl Related<super::example::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Example.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

pub struct PackageToExamples;

impl Linked for PackageToExamples {
    type FromEntity = super::package::Entity;
    type ToEntity = super::example::Entity;

    fn link(&self) -> Vec<RelationDef> {
        vec![Relation::Package.def().rev(), Relation::Example.def()]
    }
}

pub struct ExampleToPackages;

impl Linked for ExampleToPackages {
    type FromEntity = super::example::Entity;
    type ToEntity = super::package::Entity;

    fn link(&self) -> Vec<RelationDef> {
        vec![Relation::Example.def().rev(), Relation::Package.def()]
    }
}
