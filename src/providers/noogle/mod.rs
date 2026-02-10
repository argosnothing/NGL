mod schema;

use crate::{
    db::{
        entities::{example, function},
        enums::{documentation_format::DocumentationFormat, language::Language},
        services::{insert_examples, insert_functions},
    },
    providers::{Provider, ProviderInformation, noogle::schema::NoogleResponse},
    schema::{NGLDataKind, NGLRequest},
};
use pulldown_cmark::{CodeBlockKind, Event, Parser, Tag, TagEnd};
use sea_orm::{ActiveValue::*, DatabaseConnection, DbErr};

static ENDPOINT_URL: &str = "https://noogle.dev/api/v1/data";
pub struct Noogle {}

impl Provider for Noogle {
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
            let content = doc
                .content
                .as_ref()
                .and_then(|c| c.content.as_ref())
                .cloned()
                .unwrap_or_default();

            if need_functions {
                // We simply push all functions, even those
                // without descriptions, signatures, and even data.
                // The reason is sometimes it can be useful just
                // knowing the function even exists, even if
                // that means only the name is coming through.
                functions.push(function::ActiveModel {
                    id: NotSet,
                    name: Set(doc.meta.title.clone()),
                    provider_name: Set("noogle".to_owned()),
                    format: Set(DocumentationFormat::Markdown),
                    signature: Set(doc.meta.signature.clone()),
                    data: Set(content.clone()),
                });
            }

            // Examples are exclusively code blocks, content is where a code block
            // is stored, so no content == no code block :)
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

    fn get_info() -> ProviderInformation {
        ProviderInformation {
            name: "noogle".to_string(),
            source: "https://noogle.dev".to_string(),
            kinds: vec![NGLDataKind::Function, NGLDataKind::Example],
        }
    }
}
