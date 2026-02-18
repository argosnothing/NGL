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
                    .col(string_null(Example::SourceKind))
                    .col(string_null(Example::SourceLink))
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
                    .col(string_null(Function::SourceUrl))
                    .col(string_null(Function::SourceCodeUrl))
                    .col(string_null(Function::Aliases))
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
                    .col(string(Guide::Link))
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
                    .table(Alias::new("guide_xrefs"))
                    .if_not_exists()
                    .col(integer(GuideXref::GuideId))
                    .col(integer(GuideXref::SubGuideId))
                    .primary_key(
                        Index::create()
                            .col(GuideXref::GuideId)
                            .col(GuideXref::SubGuideId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-guide_xref-guide")
                            .from(Alias::new("guide_xrefs"), GuideXref::GuideId)
                            .to(Alias::new("guides"), Guide::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-guide_xref-sub_guide")
                            .from(Alias::new("guide_xrefs"), GuideXref::SubGuideId)
                            .to(Alias::new("guides"), Guide::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Alias::new("guide_examples"))
                    .if_not_exists()
                    .col(integer(GuideExample::GuideId))
                    .col(integer(GuideExample::ExampleId))
                    .col(string(GuideExample::PlaceholderKey))
                    .primary_key(
                        Index::create()
                            .col(GuideExample::GuideId)
                            .col(GuideExample::ExampleId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-guide_example-guide")
                            .from(Alias::new("guide_examples"), GuideExample::GuideId)
                            .to(Alias::new("guides"), Guide::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-guide_example-example")
                            .from(Alias::new("guide_examples"), GuideExample::ExampleId)
                            .to(Alias::new("examples"), Example::Id),
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
                    .col(string_null(Package::Description))
                    .col(string_null(Package::Homepage))
                    .col(string_null(Package::License))
                    .col(string_null(Package::SourceCodeUrl))
                    .col(boolean(Package::Broken))
                    .col(boolean(Package::Unfree))
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
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Alias::new("function_examples"))
                    .if_not_exists()
                    .col(integer(FunctionExample::FunctionId))
                    .col(integer(FunctionExample::ExampleId))
                    .col(string(FunctionExample::PlaceholderKey))
                    .primary_key(
                        Index::create()
                            .col(FunctionExample::FunctionId)
                            .col(FunctionExample::ExampleId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-function_example-function")
                            .from(Alias::new("function_examples"), FunctionExample::FunctionId)
                            .to(Alias::new("functions"), Function::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-function_example-example")
                            .from(Alias::new("function_examples"), FunctionExample::ExampleId)
                            .to(Alias::new("examples"), Example::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Alias::new("option_examples"))
                    .if_not_exists()
                    .col(integer(OptionExample::OptionId))
                    .col(integer(OptionExample::ExampleId))
                    .col(string(OptionExample::PlaceholderKey))
                    .primary_key(
                        Index::create()
                            .col(OptionExample::OptionId)
                            .col(OptionExample::ExampleId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-option_example-option")
                            .from(Alias::new("option_examples"), OptionExample::OptionId)
                            .to(Alias::new("options"), Option::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-option_example-example")
                            .from(Alias::new("option_examples"), OptionExample::ExampleId)
                            .to(Alias::new("examples"), Example::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Alias::new("package_examples"))
                    .if_not_exists()
                    .col(integer(PackageExample::PackageId))
                    .col(integer(PackageExample::ExampleId))
                    .col(string(PackageExample::PlaceholderKey))
                    .primary_key(
                        Index::create()
                            .col(PackageExample::PackageId)
                            .col(PackageExample::ExampleId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-package_example-package")
                            .from(Alias::new("package_examples"), PackageExample::PackageId)
                            .to(Alias::new("packages"), Package::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-package_example-example")
                            .from(Alias::new("package_examples"), PackageExample::ExampleId)
                            .to(Alias::new("examples"), Example::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Alias::new("type_examples"))
                    .if_not_exists()
                    .col(integer(TypeExample::TypeId))
                    .col(integer(TypeExample::ExampleId))
                    .col(string(TypeExample::PlaceholderKey))
                    .primary_key(
                        Index::create()
                            .col(TypeExample::TypeId)
                            .col(TypeExample::ExampleId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-type_example-type")
                            .from(Alias::new("type_examples"), TypeExample::TypeId)
                            .to(Alias::new("types"), Type::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-type_example-example")
                            .from(Alias::new("type_examples"), TypeExample::ExampleId)
                            .to(Alias::new("examples"), Example::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE VIRTUAL TABLE IF NOT EXISTS ngl_search USING fts5(
                    entity_id,
                    kind,
                    provider_name,
                    title,
                    content,
                    tokenize = 'ascii'
                )
                "#,
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS ngl_search")
            .await?;

        manager
            .drop_table(Table::drop().table(Alias::new("type_examples")).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Alias::new("types")).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Alias::new("package_examples")).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Alias::new("packages")).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Alias::new("option_examples")).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Alias::new("options")).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Alias::new("guide_examples")).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Alias::new("guide_xrefs")).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Alias::new("guides")).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Alias::new("function_examples")).to_owned())
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
    #[allow(unused)]
    Table,
    Name,
    LastUpdated,
}

#[derive(DeriveIden)]
#[sea_orm(iden = "provider_kind_cache")]
enum ProviderKindCache {
    #[allow(unused)]
    Table,
    ProviderName,
    Kind,
    LastSynced,
}

#[derive(DeriveIden)]
#[sea_orm(iden = "examples")]
enum Example {
    #[allow(unused)]
    Table,
    Id,
    ProviderName,
    Language,
    Data,
    SourceKind,
    SourceLink,
}

#[derive(DeriveIden)]
#[sea_orm(iden = "functions")]
enum Function {
    #[allow(unused)]
    Table,
    Id,
    Name,
    Signature,
    ProviderName,
    Format,
    Data,
    SourceUrl,
    SourceCodeUrl,
    Aliases,
}

#[derive(DeriveIden)]
#[sea_orm(iden = "guides")]
enum Guide {
    #[allow(unused)]
    Table,
    Id,
    Link,
    ProviderName,
    Title,
    Format,
    Data,
}

#[derive(DeriveIden)]
#[sea_orm(iden = "guide_xrefs")]
enum GuideXref {
    #[allow(unused)]
    Table,
    GuideId,
    SubGuideId,
}

#[derive(DeriveIden)]
#[sea_orm(iden = "guide_examples")]
enum GuideExample {
    #[allow(unused)]
    Table,
    GuideId,
    ExampleId,
    PlaceholderKey,
}

#[derive(DeriveIden)]
#[sea_orm(iden = "options")]
enum Option {
    #[allow(unused)]
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
    #[allow(unused)]
    Table,
    Id,
    ProviderName,
    Name,
    Version,
    Format,
    Data,
    Description,
    Homepage,
    License,
    SourceCodeUrl,
    Broken,
    Unfree,
}

#[derive(DeriveIden)]
#[sea_orm(iden = "types")]
enum Type {
    #[allow(unused)]
    Table,
    Id,
    ProviderName,
    Name,
    Format,
    Data,
}

#[derive(DeriveIden)]
#[sea_orm(iden = "function_examples")]
enum FunctionExample {
    #[allow(unused)]
    Table,
    FunctionId,
    ExampleId,
    PlaceholderKey,
}

#[derive(DeriveIden)]
#[sea_orm(iden = "option_examples")]
enum OptionExample {
    #[allow(unused)]
    Table,
    OptionId,
    ExampleId,
    PlaceholderKey,
}

#[derive(DeriveIden)]
#[sea_orm(iden = "package_examples")]
enum PackageExample {
    #[allow(unused)]
    Table,
    PackageId,
    ExampleId,
    PlaceholderKey,
}

#[derive(DeriveIden)]
#[sea_orm(iden = "type_examples")]
enum TypeExample {
    #[allow(unused)]
    Table,
    TypeId,
    ExampleId,
    PlaceholderKey,
}
