use sea_orm::{DeriveActiveEnum, EnumIter};

#[derive(Clone, Debug, PartialEq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Text")]
pub enum DocumentationFormat {
    #[sea_orm(string_value = "html")]
    HTML,
    #[sea_orm(string_value = "markdown")]
    Markdown,
    #[sea_orm(string_value = "plaintext")]
    PlainText,
}
