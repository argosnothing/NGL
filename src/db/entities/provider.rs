use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "providers")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub name: String,
    pub last_updated: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::example::Entity")]
    Examples,
}

impl Related<super::example::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Examples.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
