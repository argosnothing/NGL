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

impl std::fmt::Display for DocumentationFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DocumentationFormat::HTML => write!(f, "html"),
            DocumentationFormat::Markdown => write!(f, "markdown"),
            DocumentationFormat::PlainText => write!(f, "plaintext"),
        }
    }
}
