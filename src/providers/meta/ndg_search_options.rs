use sea_orm::DbErr;
use serde::Deserialize;

use crate::{
    providers::{
        EventChannel, ProviderInformation,
        meta::{ConfigProvider, TemplateProviderConfig},
    },
    utils::fetch_source,
};

pub struct NdgSearchOptionProvider {
    info: ProviderInformation,
}

impl NdgSearchOptionProvider {
    pub fn from_config(cfg: &TemplateProviderConfig) -> Self {
        Self {
            info: cfg.to_provider_info(Some(&["option", "options"])),
        }
    }

    async fn parse_options(&self, channel: &EventChannel) -> Result<(), DbErr> {
        let json_str = fetch_source(&self.info.source)
            .await
            .map_err(|e| DbErr::Custom(format!("Failed to fetch source: {}", e)))?;
        let options: Vec<OptionEntry> = serde_json::from_str(&json_str)
            .map_err(|e| DbErr::Custom(format!("Failed to parse options.json: {}", e)))?;
        println!("{:#?}", &options[..options.len().min(3)]);
        return Ok(());
    }
}

impl ConfigProvider for NdgSearchOptionProvider {
    fn provider_info(&self) -> &ProviderInformation {
        &self.info
    }

    async fn sync(
        &mut self,
        channel: &EventChannel,
        kinds: &[crate::NGLDataKind],
    ) -> Result<(), DbErr> {
        if kinds.contains(&crate::NGLDataKind::Option)
            && self.info.kinds.contains(&crate::NGLDataKind::Option)
        {
            self.parse_options(channel).await?
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize, serde::Serialize)]
pub struct OptionEntry {
    pub id: String,
    pub title: String,
    pub content: String,
    pub path: String,
    pub tokens: Vec<String>,
    pub title_tokens: Vec<String>,
}
