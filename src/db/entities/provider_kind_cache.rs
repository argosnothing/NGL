use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "provider_kind_cache")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub provider_name: String,
    #[sea_orm(primary_key, auto_increment = false)]
    pub kind: String,
    pub last_synced: DateTimeWithTimeZone,
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
