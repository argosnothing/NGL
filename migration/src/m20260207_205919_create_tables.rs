use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create providers table
        manager
            .create_table(
                Table::create()
                    .table(Provider::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Provider::Name)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Provider::LastUpdated)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Create examples table
        manager
            .create_table(
                Table::create()
                    .table(Example::Table)
                    .if_not_exists()
                    .col(pk_auto(Example::Id))
                    .col(string(Example::ProviderName))
                    .col(string(Example::Data))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-example-provider")
                            .from(Example::Table, Example::ProviderName)
                            .to(Provider::Table, Provider::Name),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop in reverse order (examples first due to foreign key)
        manager
            .drop_table(Table::drop().table(Example::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Provider::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Provider {
    Table,
    Name,
    LastUpdated,
}

#[derive(DeriveIden)]
enum Example {
    Table,
    Id,
    ProviderName,
    Data,
}
