use sea_orm::{DeriveActiveEnum, EnumIter};
use std::fmt;

#[derive(Clone, Debug, PartialEq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Text")]
pub enum Language {
    #[sea_orm(string_value = "nix")]
    Nix,
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Language::Nix => write!(f, "nix"),
        }
    }
}
