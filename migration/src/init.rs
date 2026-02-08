use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
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

        manager
            .create_table(
                Table::create()
                    .table(Example::Table)
                    .if_not_exists()
                    .col(pk_auto(Example::Id))
                    .col(string(Example::ProviderName))
                    .col(string(Example::Language))
                    .col(string(Example::Data))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-example-provider")
                            .from(Example::Table, Example::ProviderName)
                            .to(Provider::Table, Provider::Name),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Function::Table)
                    .if_not_exists()
                    .col(pk_auto(Function::Id))
                    .col(string(Function::ProviderName))
                    .col(string(Function::Format))
                    .col(string(Function::Data))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-function-provider")
                            .from(Function::Table, Function::ProviderName)
                            .to(Provider::Table, Provider::Name),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Function::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Example::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Provider::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
#[sea_orm(iden = "providers")]
enum Provider {
    Table,
    Name,
    LastUpdated,
}

#[derive(DeriveIden)]
#[sea_orm(iden = "examples")]
enum Example {
    Table,
    Id,
    ProviderName,
    Language,
    Data,
}

#[derive(DeriveIden)]
#[sea_orm(iden = "functions")]
enum Function {
    Table,
    Id,
    ProviderName,
    Format,
    Data,
}
