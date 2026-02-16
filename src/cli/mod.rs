pub mod progress;

use clap::{Parser, ValueEnum};

use crate::schema::{NGLDataKind, NGLRequest};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Kind {
    Function,
    Example,
    Guide,
    Option,
    Package,
    Type,
}

impl From<Kind> for NGLDataKind {
    fn from(kind: Kind) -> Self {
        match kind {
            Kind::Function => NGLDataKind::Function,
            Kind::Example => NGLDataKind::Example,
            Kind::Guide => NGLDataKind::Guide,
            Kind::Option => NGLDataKind::Option,
            Kind::Package => NGLDataKind::Package,
            Kind::Type => NGLDataKind::Type,
        }
    }
}

#[derive(Parser, Debug)]
#[command(name = "ngl")]
#[command(about = "Nix Global Lookup", long_about = None)]
pub struct Cli {
    pub search_term: Option<String>,

    #[arg(short, long, value_delimiter = ',')]
    pub providers: Option<Vec<String>>,

    #[arg(short, long, value_delimiter = ',')]
    pub kinds: Option<Vec<Kind>>,

    #[arg(long, env = "DATABASE_URL")]
    pub database_url: Option<String>,
}

impl From<Cli> for NGLRequest {
    fn from(cli: Cli) -> Self {
        NGLRequest {
            search_term: cli.search_term,
            providers: cli.providers,
            kinds: cli.kinds.map(|k| k.into_iter().map(Into::into).collect()),
        }
    }
}
