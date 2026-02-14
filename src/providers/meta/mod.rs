// Providers that provide... Providers!!!!
//
// Massive credit to nix-search-tv for the idea of config based templates.
// https://github.com/3timeslazy/nix-search-tv
use crate::providers::Provider;
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

pub fn is_url(source: &str) -> bool {
    source.starts_with("http://") || source.starts_with("https://")
}

pub fn html_to_markdown(html: &str) -> String {
    html2md::parse_html(html)
}

pub async fn fetch_source(
    source: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    if is_url(source) {
        let resp = reqwest::get(source).await?;
        if !resp.status().is_success() {
            return Err(format!("HTTP error: {}", resp.status()).into());
        }
        Ok(resp.text().await?)
    } else {
        Ok(tokio::fs::read_to_string(source).await?)
    }
}
