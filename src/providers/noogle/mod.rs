mod schema;

use crate::{
    db::{
        entities::{example, function},
        enums::{documentation_format::DocumentationFormat, language::Language},
    },
    providers::{Provider, ProviderInformation, noogle::schema::NoogleResponse},
    schema::NGLDataKind,
};
use async_trait::async_trait;
use pulldown_cmark::{CodeBlockKind, Event, Parser, Tag, TagEnd};
use sea_orm::ActiveValue::*;

static ENDPOINT_URL: &str = "https://noogle.dev/api/v1/data";

pub struct Noogle {
    cached_response: Option<NoogleResponse>,
    cached_functions: Option<Vec<function::ActiveModel>>,
    cached_examples: Option<Vec<example::ActiveModel>>,
}

impl Noogle {
    pub fn new() -> Self {
        Self {
            cached_response: None,
            cached_functions: None,
            cached_examples: None,
        }
    }

    async fn fetch(&mut self) -> &NoogleResponse {
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

    async fn process_data(&mut self) {
        if self.cached_functions.is_some() && self.cached_examples.is_some() {
            return;
        }

        let response = self.fetch().await;
        let mut functions = Vec::new();
        let mut examples = Vec::new();

        for doc in &response.data {
            let content = doc
                .content
                .as_ref()
                .and_then(|c| c.content.as_ref())
                .cloned()
                .unwrap_or_default();

            functions.push(function::ActiveModel {
                id: NotSet,
                name: Set(doc.meta.title.clone()),
                provider_name: Set("noogle".to_owned()),
                format: Set(DocumentationFormat::Markdown),
                signature: Set(doc.meta.signature.clone()),
                data: Set(content.clone()),
            });

            if !content.is_empty() {
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
        }

        self.cached_functions = Some(functions);
        self.cached_examples = Some(examples);
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
        self.process_data().await;
        self.cached_functions.clone().unwrap()
    }

    async fn fetch_examples(&mut self) -> Vec<example::ActiveModel> {
        self.process_data().await;
        self.cached_examples.clone().unwrap()
    }

    // noogle
    async fn fetch_guides(&mut self) -> Vec<crate::db::entities::guide::ActiveModel> {
        vec![]
    }

    // only
    async fn fetch_options(&mut self) -> Vec<crate::db::entities::option::ActiveModel> {
        vec![]
    }

    // does
    async fn fetch_packages(&mut self) -> Vec<crate::db::entities::package::ActiveModel> {
        vec![]
    }

    // functions!
    async fn fetch_types(&mut self) -> Vec<crate::db::entities::r#type::ActiveModel> {
        vec![]
    }
}
