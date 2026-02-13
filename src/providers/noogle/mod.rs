mod schema;

use crate::{
    db::{
        entities::{example, function},
        enums::{documentation_format::DocumentationFormat, language::Language},
    },
    providers::{Provider, ProviderEvent, ProviderInformation, Sink, noogle::schema::NoogleResponse},
    schema::NGLDataKind,
};
use async_trait::async_trait;
use pulldown_cmark::{CodeBlockKind, Event, Parser, Tag, TagEnd};
use sea_orm::{ActiveValue::*, DbErr};
use std::sync::Arc;

static ENDPOINT_URL: &str = "https://noogle.dev/api/v1/data";

pub struct Noogle;

impl Noogle {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Provider for Noogle {
    fn get_info(&self) -> ProviderInformation {
        ProviderInformation {
            name: "noogle".to_string(),
            source: "https://noogle.dev".to_string(),
            kinds: vec![NGLDataKind::Function, NGLDataKind::Example],
        }
    }

    async fn sync(&mut self, sink: Arc<dyn Sink>, kinds: &[NGLDataKind]) -> Result<(), DbErr> {
        let response = reqwest::get(ENDPOINT_URL)
            .await
            .map_err(|e| DbErr::Custom(e.to_string()))?
            .json::<NoogleResponse>()
            .await
            .map_err(|e| DbErr::Custom(e.to_string()))?;

        let fetch_functions = kinds.contains(&NGLDataKind::Function);
        let fetch_examples = kinds.contains(&NGLDataKind::Example);

        for doc in response.data {
            let content = doc
                .content
                .as_ref()
                .and_then(|c| c.content.as_ref())
                .cloned()
                .unwrap_or_default();

            if fetch_functions {
                sink.emit(ProviderEvent::Function(function::ActiveModel {
                    id: NotSet,
                    name: Set(doc.meta.title.clone()),
                    provider_name: Set("noogle".to_owned()),
                    format: Set(DocumentationFormat::Markdown),
                    signature: Set(doc.meta.signature.clone()),
                    data: Set(content.clone()),
                }))
                .await?;
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
                                sink.emit(ProviderEvent::Example(example::ActiveModel {
                                    id: NotSet,
                                    provider_name: Set("noogle".to_owned()),
                                    language: Set(Some(Language::Nix)),
                                    data: Set(current_code.clone()),
                                }))
                                .await?;
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
