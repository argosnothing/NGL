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
                            .from(
                                Alias::new("provider_kind_cache"),
                                ProviderKindCache::ProviderName,
                            )
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
                    .col(string(Function::Name))
                    .col(string(Function::ProviderName))
                    .col(string_null(Function::Signature))
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
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Alias::new("guides"))
                    .if_not_exists()
                    .col(pk_auto(Guide::Id))
                    .col(string(Guide::ProviderName))
                    .col(string(Guide::Title))
                    .col(string(Guide::Format))
                    .col(string(Guide::Data))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-guide-provider")
                            .from(Alias::new("guides"), Guide::ProviderName)
                            .to(Alias::new("providers"), Provider::Name),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Alias::new("options"))
                    .if_not_exists()
                    .col(pk_auto(Option::Id))
                    .col(string(Option::ProviderName))
                    .col(string(Option::Name))
                    .col(string_null(Option::TypeSignature))
                    .col(string_null(Option::DefaultValue))
                    .col(string(Option::Format))
                    .col(string(Option::Data))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-option-provider")
                            .from(Alias::new("options"), Option::ProviderName)
                            .to(Alias::new("providers"), Provider::Name),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Alias::new("packages"))
                    .if_not_exists()
                    .col(pk_auto(Package::Id))
                    .col(string(Package::ProviderName))
                    .col(string(Package::Name))
                    .col(string_null(Package::Version))
                    .col(string(Package::Format))
                    .col(string(Package::Data))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-package-provider")
                            .from(Alias::new("packages"), Package::ProviderName)
                            .to(Alias::new("providers"), Provider::Name),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Alias::new("types"))
                    .if_not_exists()
                    .col(pk_auto(Type::Id))
                    .col(string(Type::ProviderName))
                    .col(string(Type::Name))
                    .col(string(Type::Format))
                    .col(string(Type::Data))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-type-provider")
                            .from(Alias::new("types"), Type::ProviderName)
                            .to(Alias::new("providers"), Provider::Name),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Alias::new("types")).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Alias::new("packages")).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Alias::new("options")).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Alias::new("guides")).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Alias::new("functions")).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Alias::new("examples")).to_owned())
            .await?;

        manager
            .drop_table(
                Table::drop()
                    .table(Alias::new("provider_kind_cache"))
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(
                Table::drop()
                    .table(Alias::new("provider_kind_cache"))
                    .to_owned(),
            )
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
    Name,
    Signature,
    ProviderName,
    Format,
    Data,
}

#[derive(DeriveIden)]
#[sea_orm(iden = "guides")]
enum Guide {
    Table,
    Id,
    ProviderName,
    Title,
    Format,
    Data,
}

#[derive(DeriveIden)]
#[sea_orm(iden = "options")]
enum Option {
    Table,
    Id,
    ProviderName,
    Name,
    TypeSignature,
    DefaultValue,
    Format,
    Data,
}

#[derive(DeriveIden)]
#[sea_orm(iden = "packages")]
enum Package {
    Table,
    Id,
    ProviderName,
    Name,
    Version,
    Format,
    Data,
}

#[derive(DeriveIden)]
#[sea_orm(iden = "types")]
enum Type {
    Table,
    Id,
    ProviderName,
    Name,
    Format,
    Data,
}
