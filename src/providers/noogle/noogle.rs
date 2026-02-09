use super::schema::NoogleResponse;
use crate::{
    db::{
        entities::{example, function},
        enums::{documentation_format::DocumentationFormat, language::Language},
        services::services::{insert_examples, insert_functions},
    },
    providers::Provider,
    schema::{NGLDataKind, NGLRequest},
};
use pulldown_cmark::{CodeBlockKind, Event, Parser, Tag, TagEnd};
use sea_orm::{ActiveValue::*, DatabaseConnection, DbErr};

pub struct Noogle {}
static ENDPOINT_URL: &str = "https://noogle.dev/api/v1/data";

impl Provider for Noogle {
    fn get_supported_kinds() -> Vec<NGLDataKind> {
        vec![NGLDataKind::Function, NGLDataKind::Example]
    }

    async fn fetch_and_insert(db: &DatabaseConnection, request: NGLRequest) -> Result<(), DbErr> {
        let response = reqwest::get(ENDPOINT_URL)
            .await
            .expect("Failed to fetch from Noogle")
            .json::<NoogleResponse>()
            .await
            .expect("Failed to parse Noogle response");

        let kinds = request.kinds.as_ref().unwrap();
        let need_functions = kinds.contains(&NGLDataKind::Function);
        let need_examples = kinds.contains(&NGLDataKind::Example);

        let mut functions = Vec::new();
        let mut examples = Vec::new();

        for doc in &response.data {
            if doc.meta.signature.is_none() {
                continue;
            }

            let content = doc
                .content
                .as_ref()
                .and_then(|c| c.content.as_ref())
                .cloned()
                .unwrap_or_default();

            if need_functions {
                functions.push(function::ActiveModel {
                    id: NotSet,
                    name: Set(doc.meta.title.clone()),
                    provider_name: Set("noogle".to_owned()),
                    format: Set(DocumentationFormat::Markdown),
                    signature: Set(doc.meta.signature.clone().unwrap()),
                    data: Set(content.clone()),
                });
            }

            if need_examples && !content.is_empty() {
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

        if !functions.is_empty() {
            insert_functions(db, functions).await?;
        }

        if !examples.is_empty() {
            insert_examples(db, examples).await?;
        }

        Ok(())
    }

    fn get_name() -> String {
        "noogle".to_owned()
    }
}
