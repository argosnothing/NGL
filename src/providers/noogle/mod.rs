mod schema;

use crate::{
    db::{
        entities::{example, function},
        enums::{documentation_format::DocumentationFormat, language::Language},
    },
    providers::{
        Provider, ProviderEvent, ProviderInformation, EventChannel, noogle::schema::NoogleResponse,
    },
    schema::NGLDataKind,
};
use async_trait::async_trait;
use pulldown_cmark::{CodeBlockKind, Event, Parser, Tag, TagEnd};
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

            if fetch_functions {
                let source_url = Some(format!("https://noogle.dev/f/{}", doc.meta.path.join("/")));

                let source_code_url = doc.meta.attr_position.as_ref().map(|pos| {
                    format!(
                        "https://github.com/NixOS/nixpkgs/blob/{}/{}#L{}",
                        upstream_rev, pos.file, pos.line
                    )
                });

                let aliases = doc.meta.aliases.as_ref().map(|a| {
                    serde_json::to_string(
                        &a.iter().map(|parts| parts.join(".")).collect::<Vec<_>>(),
                    )
                    .unwrap_or_default()
                });

                channel.send(ProviderEvent::Function(function::ActiveModel {
                    id: NotSet,
                    name: Set(doc.meta.title.clone()),
                    provider_name: Set("noogle".to_owned()),
                    format: Set(DocumentationFormat::Markdown),
                    signature: Set(doc.meta.signature.clone()),
                    data: Set(content.clone()),
                    source_url: Set(source_url),
                    source_code_url: Set(source_code_url),
                    aliases: Set(aliases),
                })).await;
            }

            if fetch_examples && !content.is_empty() {
                let parser = Parser::new(content.as_str());
                let mut in_code_block = false;
                let mut current_lang: Option<String> = None;
                let mut current_code = String::new();

                for event in parser {
                    match event {
                        Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang))) => {
                            in_code_block = true;
                            current_lang = Some(lang.to_string());
                            current_code.clear();
                        }
                        Event::Text(text) if in_code_block => {
                            current_code.push_str(&text);
                        }
                        Event::End(TagEnd::CodeBlock) if in_code_block => {
                            in_code_block = false;

                            if current_lang.as_deref() == Some("nix")
                                && !current_code.trim().is_empty()
                            {
                                channel.send(ProviderEvent::Example(example::ActiveModel {
                                    id: NotSet,
                                    provider_name: Set("noogle".to_owned()),
                                    language: Set(Some(Language::Nix)),
                                    data: Set(current_code.clone()),
                                })).await;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(())
    }
}
