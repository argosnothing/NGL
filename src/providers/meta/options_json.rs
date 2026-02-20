use crate::db::entities::option as option_entity;
use crate::db::enums::documentation_format::DocumentationFormat;
use crate::providers::{EventChannel, ProviderEvent, ProviderInformation};
use crate::schema::NGLDataKind;
use crate::utils::fetch_source;
use sea_orm::ActiveValue::*;
use sea_orm::DbErr;
use serde::Deserialize;
use std::collections::HashMap;

use super::{ConfigProvider, TemplateProviderConfig};

pub struct OptionsJsonProvider {
    info: ProviderInformation,
}

impl OptionsJsonProvider {
    pub fn from_config(cfg: &TemplateProviderConfig) -> Self {
        Self {
            info: cfg.to_provider_info(Some(&["option", "options"])),
        }
    }

    async fn parse_options(&self, channel: &EventChannel) -> Result<(), DbErr> {
        let json_str = fetch_source(&self.info.source)
            .await
            .map_err(|e| DbErr::Custom(format!("Failed to fetch source: {}", e)))?;

        let options: HashMap<String, OptionEntry> = serde_json::from_str(&json_str)
            .map_err(|e| DbErr::Custom(format!("Failed to parse options.json: {}", e)))?;

        for (name, opt) in options {
            let default_value = opt.default.as_ref().map(|d| {
                if let Some(text) = d.text() {
                    text.clone()
                } else {
                    serde_json::to_string(d).unwrap_or_default()
                }
            });

            let data = serde_json::to_string(&opt).unwrap_or_default();

            channel
                .send(ProviderEvent::Option(option_entity::ActiveModel {
                    id: NotSet,
                    provider_name: Set(self.info.name.clone()),
                    name: Set(name.clone()),
                    type_signature: Set(opt.option_type),
                    default_value: Set(default_value),
                    format: Set(DocumentationFormat::Markdown),
                    data: Set(data),
                }))
                .await;
        }

        Ok(())
    }
}

impl ConfigProvider for OptionsJsonProvider {
    fn provider_info(&self) -> &ProviderInformation {
        &self.info
    }

    async fn sync(&mut self, channel: &EventChannel, kinds: &[NGLDataKind]) -> Result<(), DbErr> {
        if kinds.contains(&NGLDataKind::Option) && self.info.kinds.contains(&NGLDataKind::Option) {
            self.parse_options(channel).await?
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize, serde::Serialize)]
pub struct OptionEntry {
    #[serde(rename = "type")]
    pub option_type: Option<String>,

    pub description: Option<String>,

    #[serde(rename = "default")]
    pub default: Option<OptionValue>,

    pub example: Option<OptionValue>,

    pub declarations: Option<Vec<serde_json::Value>>,

    #[serde(rename = "readOnly")]
    pub read_only: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum OptionValue {
    Text { text: Option<String> },
    Raw(serde_json::Value),
}

impl OptionValue {
    pub fn text(&self) -> Option<&String> {
        match self {
            OptionValue::Text { text } => text.as_ref(),
            OptionValue::Raw(_) => None,
        }
    }
}

impl serde::Serialize for OptionValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            OptionValue::Text { text } => text.serialize(serializer),
            OptionValue::Raw(v) => v.serialize(serializer),
        }
    }
}
