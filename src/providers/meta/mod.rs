// Providers that provide... Providers!!!!
//
// Massive credit to nix-search-tv for the idea of config based templates.
// https://github.com/3timeslazy/nix-search-tv
use crate::providers::Sink;
use crate::providers::{Provider, ProviderInformation};
use crate::schema::NGLDataKind;
use serde::Deserialize;
use std::path::PathBuf;

mod ndg_options_html;
mod options_json;
mod renderdocs;

pub use ndg_options_html::NdgOptionsHtmlProvider;
pub use options_json::OptionsJsonProvider;
pub use renderdocs::RenderDocsProvider;

#[derive(Debug, Clone, Deserialize)]
pub struct TemplateProviderConfig {
    pub template: String,
    pub name: String,
    pub source: String,
    pub kinds: Vec<String>,
}

impl TemplateProviderConfig {
    pub fn to_provider_info(&self, allowed_kinds: Option<&[&str]>) -> ProviderInformation {
        let kinds = self
            .kinds
            .iter()
            .filter_map(|k| {
                let lower = k.to_lowercase();
                let kind = match lower.as_str() {
                    "option" | "options" => Some(NGLDataKind::Option),
                    "function" | "functions" => Some(NGLDataKind::Function),
                    "example" | "examples" => Some(NGLDataKind::Example),
                    "guide" | "guides" => Some(NGLDataKind::Guide),
                    _ => None,
                };

                if let Some(allowed) = allowed_kinds {
                    if !allowed.contains(&lower.as_str())
                        && !allowed.iter().any(|a| lower.starts_with(a))
                    {
                        eprintln!("Warning: kind '{}' not supported for this template", k);
                        return None;
                    }
                }

                if kind.is_none() {
                    eprintln!("Warning: unknown kind '{}'", k);
                }

                kind
            })
            .collect();

        ProviderInformation {
            name: self.name.clone(),
            source: self.source.clone(),
            kinds,
            sync_interval_hours: Some(24),
        }
    }
}

pub trait ConfigProvider: Send {
    fn provider_info(&self) -> &ProviderInformation;

    fn sync(
        &mut self,
        sink: &dyn Sink,
        kinds: &[NGLDataKind],
    ) -> impl std::future::Future<Output = Result<(), sea_orm::DbErr>> + Send;
}

#[async_trait::async_trait]
impl<T: ConfigProvider> Provider for T {
    fn get_info(&self) -> ProviderInformation {
        self.provider_info().clone()
    }

    async fn sync(
        &mut self,
        sink: &dyn Sink,
        kinds: &[NGLDataKind],
    ) -> Result<(), sea_orm::DbErr> {
        ConfigProvider::sync(self, sink, kinds).await
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct MetaProviderConfig {
    #[serde(default)]
    pub template_providers: Vec<TemplateProviderConfig>,
}

impl MetaProviderConfig {
    pub fn from_file(path: &PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = std::fs::read_to_string(path)?;
        let config: MetaProviderConfig = serde_json::from_str(&contents)?;
        Ok(config)
    }

    #[allow(unused)]
    pub fn from_str(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

pub struct MetaProvider {
    config: MetaProviderConfig,
}

impl MetaProvider {
    pub fn new(config: MetaProviderConfig) -> Self {
        Self { config }
    }

    pub fn from_file(path: &PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let config = MetaProviderConfig::from_file(path)?;
        Ok(Self::new(config))
    }

    pub fn build_providers(&self) -> Vec<Box<dyn Provider + Send>> {
        self.config
            .template_providers
            .iter()
            .filter_map(|cfg| self.build_provider(cfg))
            .collect()
    }

    fn build_provider(&self, cfg: &TemplateProviderConfig) -> Option<Box<dyn Provider + Send>> {
        match cfg.template.as_str() {
            "renderdocs" => Some(Box::new(RenderDocsProvider::from_config(cfg))),
            "options_json" => Some(Box::new(OptionsJsonProvider::from_config(cfg))),
            "ndg_options_html" => Some(Box::new(NdgOptionsHtmlProvider::from_config(cfg))),
            unknown => {
                eprintln!("Warning: unknown template type '{}', skipping", unknown);
                None
            }
        }
    }
}
