use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("providers"))
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
                    .table(Alias::new("provider_kind_cache"))
                    .if_not_exists()
                    .col(string(ProviderKindCache::ProviderName))
                    .col(string(ProviderKindCache::Kind))
                    .col(timestamp_with_time_zone(ProviderKindCache::LastSynced))
                    .primary_key(
                        Index::create()
                            .col(ProviderKindCache::ProviderName)
                            .col(ProviderKindCache::Kind),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-provider_kind_cache-provider")
                            .from(Alias::new("provider_kind_cache"), ProviderKindCache::ProviderName)
                            .to(Alias::new("providers"), Provider::Name),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Alias::new("examples"))
                    .if_not_exists()
                    .col(pk_auto(Example::Id))
                    .col(string(Example::ProviderName))
                    .col(string_null(Example::Language))
                    .col(string(Example::Data))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-example-provider")
                            .from(Alias::new("examples"), Example::ProviderName)
                            .to(Alias::new("providers"), Provider::Name),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Alias::new("functions"))
                    .if_not_exists()
                    .col(pk_auto(Function::Id))
                    .col(string(Function::ProviderName))
                    .col(string(Function::Signature))
                    .col(string(Function::Format))
                    .col(string(Function::Data))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-function-provider")
                            .from(Alias::new("functions"), Function::ProviderName)
                            .to(Alias::new("providers"), Provider::Name),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Alias::new("functions")).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Alias::new("examples")).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Alias::new("provider_kind_cache")).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Alias::new("provider_kind_cache")).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Alias::new("providers")).to_owned())
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
#[sea_orm(iden = "provider_kind_cache")]
enum ProviderKindCache {
    Table,
    ProviderName,
    Kind,
    LastSynced,
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
    Signature,
    ProviderName,
    Format,
    Data,
}
