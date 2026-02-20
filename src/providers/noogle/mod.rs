mod schema;

use crate::{
    db::{
        entities::{example, function},
        enums::documentation_format::DocumentationFormat,
    },
    providers::{
        EventChannel, Provider, ProviderEvent, ProviderInformation, noogle::schema::NoogleResponse,
    },
    schema::NGLDataKind,
    utils::extract_examples_markdown,
};
use async_trait::async_trait;
use sea_orm::{ActiveValue::*, DbErr};

static ENDPOINT_URL: &str = "https://noogle.dev/api/v1/data";

#[derive(Default)]
pub struct Noogle;

#[async_trait]
impl Provider for Noogle {
    fn get_info(&self) -> ProviderInformation {
        ProviderInformation {
            name: "noogle".to_string(),
            source: "https://noogle.dev".to_string(),
            kinds: vec![NGLDataKind::Function, NGLDataKind::Example],
            sync_interval_hours: Some(24),
        }
    }

    async fn sync(&mut self, channel: &EventChannel, kinds: &[NGLDataKind]) -> Result<(), DbErr> {
        let response = reqwest::get(ENDPOINT_URL)
            .await
            .map_err(|e| DbErr::Custom(e.to_string()))?
            .json::<NoogleResponse>()
            .await
            .map_err(|e| DbErr::Custom(e.to_string()))?;

        let fetch_functions = kinds.contains(&NGLDataKind::Function);
        let fetch_examples = kinds.contains(&NGLDataKind::Example);
        let upstream_rev = response.upstream_info.rev.clone();

        for doc in response.data {
            let content = doc
                .content
                .as_ref()
                .and_then(|c| c.content.as_ref())
                .cloned()
                .unwrap_or_default();

            let source_url = Some(format!("https://noogle.dev/f/{}", doc.meta.path.join("/")));

            let source_code_url = doc.meta.attr_position.as_ref().map(|pos| {
                format!(
                    "https://github.com/NixOS/nixpkgs/blob/{}/{}#L{}",
                    upstream_rev, pos.file, pos.line
                )
            });
            if fetch_examples {
                let extracted_examples = extract_examples_markdown(&content);
                for example in extracted_examples {
                    channel
                        .send(ProviderEvent::Example(example::ActiveModel {
                            id: NotSet,
                            provider_name: Set("noogle".to_owned()),
                            language: Set(example.language),
                            data: Set(example.data),
                            source_kind: Set(Some(NGLDataKind::Function)),
                            source_link: Set(if doc.meta.path.is_empty() {
                                None
                            } else {
                                Some(format!(
                                    "https://github.com/NixOS/nixpkgs/blob/master/{}",
                                    doc.meta.path.join("/")
                                ))
                            }),
                        }))
                        .await
                }
            }
            if fetch_functions {
                let aliases = doc.meta.aliases.as_ref().map(|a| {
                    serde_json::to_string(
                        &a.iter().map(|parts| parts.join(".")).collect::<Vec<_>>(),
                    )
                    .unwrap_or_default()
                });

                channel
                    .send(ProviderEvent::Function(function::ActiveModel {
                        id: NotSet,
                        name: Set(doc.meta.title.clone()),
                        provider_name: Set("noogle".to_owned()),
                        format: Set(DocumentationFormat::Markdown),
                        signature: Set(doc.meta.signature.clone()),
                        data: Set(content),
                        source_url: Set(source_url),
                        source_code_url: Set(source_code_url),
                        aliases: Set(aliases),
                    }))
                    .await;
            }
        }

        Ok(())
    }
}
