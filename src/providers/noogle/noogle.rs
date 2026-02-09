use super::schema::NoogleResponse;
use crate::{
    db::{
        entities::{example, function},
        enums::{documentation_format::DocumentationFormat, language::Language},
        services::services::{insert_examples, insert_functions},
    },
    providers::{
        Provider,
        traits::{ProvidesExamples, ProvidesFunctions},
    },
    schema::{NGLDataKind, NGLRequest},
};
use pulldown_cmark::{CodeBlockKind, Event, Parser, Tag, TagEnd};
use sea_orm::{ActiveValue::*, DatabaseConnection, DbErr};

pub struct Noogle {}
static ENDPOINT_URL: &str = "https://noogle.dev/api/v1/data";

impl Provider for Noogle {
    async fn fetch_and_insert(db: &DatabaseConnection, request: NGLRequest) -> Result<(), DbErr> {
        let response = reqwest::get(ENDPOINT_URL)
            .await
            .expect("Failed to fetch from Noogle")
            .json::<NoogleResponse>()
            .await
            .expect("Failed to parse Noogle response");

        if request
            .kinds
            .as_ref()
            .map_or(false, |kinds| kinds.contains(&NGLDataKind::Function))
        {
            let functions = Self::get_functions(&response);
            insert_functions(db, functions).await?;
        }

        if request
            .kinds
            .as_ref()
            .map_or(false, |kinds| kinds.contains(&NGLDataKind::Example))
        {
            let examples = Self::get_examples(&response);
            insert_examples(db, examples).await?;
        }

        Ok(())
    }

    fn get_name() -> String {
        "noogle".to_owned()
    }
}

impl ProvidesFunctions<NoogleResponse> for Noogle {
    fn get_functions(data: &NoogleResponse) -> Vec<function::ActiveModel> {
        data.data
            .iter()
            .filter(|doc| doc.meta.signature.is_some())
            .map(|doc| {
                let content = doc
                    .content
                    .as_ref()
                    .and_then(|c| c.content.as_ref())
                    .cloned()
                    .unwrap_or_default();

                function::ActiveModel {
                    id: NotSet,
                    provider_name: Set("noogle".to_owned()),
                    format: Set(DocumentationFormat::Markdown),
                    signature: Set(doc.meta.signature.clone().unwrap()),
                    data: Set(content),
                }
            })
            .collect()
    }
}

impl ProvidesExamples<NoogleResponse> for Noogle {
    fn get_examples(data: &NoogleResponse) -> Vec<example::ActiveModel> {
        let mut examples = Vec::new();

        for doc in &data.data {
            if let Some(content) = &doc.content {
                if let Some(markdown) = &content.content {
                    let parser = Parser::new(markdown.as_str());
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
        }

        examples
    }
}
