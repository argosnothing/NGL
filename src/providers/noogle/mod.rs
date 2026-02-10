mod schema;

use async_trait::async_trait;
use crate::{
    db::{
        entities::{example, function},
        enums::{documentation_format::DocumentationFormat, language::Language},
    },
    providers::{Provider, ProviderInformation, noogle::schema::NoogleResponse},
    schema::NGLDataKind,
};
use pulldown_cmark::{CodeBlockKind, Event, Parser, Tag, TagEnd};
use sea_orm::ActiveValue::*;

static ENDPOINT_URL: &str = "https://noogle.dev/api/v1/data";

pub struct Noogle {
    cached_response: Option<NoogleResponse>,
}

impl Noogle {
    pub fn new() -> Self {
        Self {
            cached_response: None,
        }
    }

    async fn fetch_once(&mut self) -> &NoogleResponse {
        if self.cached_response.is_none() {
            let response = reqwest::get(ENDPOINT_URL)
                .await
                .expect("Failed to fetch from Noogle")
                .json::<NoogleResponse>()
                .await
                .expect("Failed to parse Noogle response");
            self.cached_response = Some(response);
        }
        self.cached_response.as_ref().unwrap()
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

    async fn fetch_functions(&mut self) -> Vec<function::ActiveModel> {
        let response = self.fetch_once().await;

        response
            .data
            .iter()
            .map(|doc| {
                let content = doc
                    .content
                    .as_ref()
                    .and_then(|c| c.content.as_ref())
                    .cloned()
                    .unwrap_or_default();

                function::ActiveModel {
                    id: NotSet,
                    name: Set(doc.meta.title.clone()),
                    provider_name: Set("noogle".to_owned()),
                    format: Set(DocumentationFormat::Markdown),
                    signature: Set(doc.meta.signature.clone()),
                    data: Set(content),
                }
            })
            .collect()
    }

    async fn fetch_examples(&mut self) -> Vec<example::ActiveModel> {
        let response = self.fetch_once().await;
        let mut examples = Vec::new();

        for doc in &response.data {
            let content = doc
                .content
                .as_ref()
                .and_then(|c| c.content.as_ref())
                .cloned()
                .unwrap_or_default();

            if content.is_empty() {
                continue;
            }

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

                        if current_lang.as_deref() == Some("nix") && !current_code.trim().is_empty()
                        {
                            examples.push(example::ActiveModel {
                                id: NotSet,
                                provider_name: Set("noogle".to_owned()),
                                language: Set(Some(Language::Nix)),
                                data: Set(current_code.clone()),
                            });
                        }
                    }
                    _ => {}
                }
            }
        }

        examples
    }

    async fn fetch_guides(&mut self) -> Vec<crate::db::entities::guide::ActiveModel> {
        vec![]
    }

    async fn fetch_options(&mut self) -> Vec<crate::db::entities::option::ActiveModel> {
        vec![]
    }

    async fn fetch_packages(&mut self) -> Vec<crate::db::entities::package::ActiveModel> {
        vec![]
    }

    async fn fetch_types(&mut self) -> Vec<crate::db::entities::r#type::ActiveModel> {
        vec![]
    }
}
