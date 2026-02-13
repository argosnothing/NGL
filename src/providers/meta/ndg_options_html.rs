use crate::db::entities::example as example_entity;
use crate::db::entities::option as option_entity;
use crate::db::enums::documentation_format::DocumentationFormat;
use crate::db::enums::language::Language;
use crate::providers::{Provider, ProviderEvent, ProviderInformation, Sink};
use crate::schema::NGLDataKind;
use async_trait::async_trait;
use scraper::{Html, Selector, ElementRef, Element};
use sea_orm::ActiveValue::*;
use sea_orm::DbErr;
use std::sync::Arc;

use super::{fetch_source, html_to_markdown, TemplateProviderConfig};

pub struct NdgOptionsHtmlProvider {
    name: String,
    source: String,
    kinds: Vec<NGLDataKind>,
}

impl NdgOptionsHtmlProvider {
    pub fn new(name: String, source: String, kinds: Vec<NGLDataKind>) -> Self {
        Self { name, source, kinds }
    }

    pub fn from_config(cfg: &TemplateProviderConfig) -> Self {
        let kinds = cfg
            .kinds
            .iter()
            .filter_map(|k| match k.to_lowercase().as_str() {
                "option" | "options" => Some(NGLDataKind::Option),
                "function" | "functions" => Some(NGLDataKind::Function),
                "example" | "examples" => Some(NGLDataKind::Example),
                "guide" | "guides" => Some(NGLDataKind::Guide),
                _ => {
                    eprintln!("Warning: unknown kind '{}' for ndg_options_html", k);
                    None
                }
            })
            .collect();

        Self::new(cfg.name.clone(), cfg.source.clone(), kinds)
    }

    async fn parse_content(&self, sink: Arc<dyn Sink>, emit_options: bool, emit_examples: bool) -> Result<(), DbErr> {
        let html = fetch_source(&self.source)
            .await
            .map_err(|e| DbErr::Custom(format!("Failed to fetch source: {}", e)))?;

        let options = parse_ndg_html(&html)
            .map_err(|e| DbErr::Custom(format!("Failed to parse HTML: {}", e)))?;

        for opt in options {
            if emit_options {
                let markdown = opt.raw_html.as_ref().map(|h| html_to_markdown(h)).unwrap_or_default();
                sink.emit(ProviderEvent::Option(option_entity::ActiveModel {
                    id: NotSet,
                    provider_name: Set(self.name.clone()),
                    name: Set(opt.name.clone()),
                    type_signature: Set(opt.option_type.clone()),
                    default_value: Set(opt.default.clone()),
                    format: Set(DocumentationFormat::Markdown),
                    data: Set(markdown),
                }))
                .await?;
            }

            if emit_examples {
                for example in &opt.examples {
                    sink.emit(ProviderEvent::Example(example_entity::ActiveModel {
                        id: NotSet,
                        provider_name: Set(self.name.clone()),
                        language: Set(Some(Language::Nix)),
                        data: Set(example.clone()),
                    }))
                    .await?;
                }
            }
        }

        Ok(())
    }
}

#[async_trait]
impl Provider for NdgOptionsHtmlProvider {
    fn get_info(&self) -> ProviderInformation {
        ProviderInformation {
            kinds: self.kinds.clone(),
            name: self.name.clone(),
            source: self.source.clone(),
        }
    }

    async fn sync(&mut self, sink: Arc<dyn Sink>, kinds: &[NGLDataKind]) -> Result<(), DbErr> {
        let emit_options = kinds.contains(&NGLDataKind::Option) && self.kinds.contains(&NGLDataKind::Option);
        let emit_examples = kinds.contains(&NGLDataKind::Example) && self.kinds.contains(&NGLDataKind::Example);
        
        if emit_options || emit_examples {
            self.parse_content(sink, emit_options, emit_examples).await?;
        }
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct ParsedOption {
    pub name: String,
    pub description: Option<String>,
    pub option_type: Option<String>,
    pub default: Option<String>,
    pub examples: Vec<String>,
    pub declared_by: Vec<String>,
    pub raw_html: Option<String>,
}

fn collect_until_next_h3<'a>(start: ElementRef<'a>) -> Vec<ElementRef<'a>> {
    let mut elements = Vec::new();
    let mut current = start;
    
    while let Some(sibling) = current.next_sibling_element() {
        if sibling.value().name() == "h3" {
            break;
        }
        elements.push(sibling);
        current = sibling;
    }
    
    elements
}

fn extract_text(element: &ElementRef) -> String {
    element.text().collect::<Vec<_>>().join("").trim().to_string()
}

fn extract_code_text(element: &ElementRef) -> Option<String> {
    let code_sel = Selector::parse("code").ok()?;
    element.select(&code_sel).next().map(|el| extract_text(&el))
}

fn extract_links(element: &ElementRef) -> Vec<String> {
    let link_sel = Selector::parse("a[href]").unwrap();
    element
        .select(&link_sel)
        .filter_map(|a| a.value().attr("href").map(|s| s.to_string()))
        .collect()
}

pub fn parse_ndg_html(html: &str) -> Result<Vec<ParsedOption>, String> {
    let document = Html::parse_document(html);
    let h3_sel = Selector::parse("h3").map_err(|e| format!("Invalid selector: {:?}", e))?;
    
    let mut options = Vec::new();

    for h3 in document.select(&h3_sel) {
        let name = extract_text(&h3)
            .replace("Link copied!", "")
            .trim()
            .to_string();
        
        if name.is_empty() || !name.contains('.') {
            continue;
        }

        let siblings = collect_until_next_h3(h3);
        
        let mut opt = ParsedOption {
            name: name.clone(),
            ..Default::default()
        };

        let mut raw_parts: Vec<String> = vec![format!("<h3>{}</h3>", name)];
        let mut description_parts: Vec<String> = Vec::new();

        for sib in &siblings {
            let text = extract_text(sib);
            let tag = sib.value().name();
            
            raw_parts.push(sib.html());

            if text.starts_with("Type:") {
                opt.option_type = extract_code_text(sib).or_else(|| {
                    Some(text.strip_prefix("Type:").unwrap_or(&text).trim().to_string())
                });
            } else if text.starts_with("Default:") {
                opt.default = extract_code_text(sib).or_else(|| {
                    Some(text.strip_prefix("Default:").unwrap_or(&text).trim().to_string())
                });
            } else if text.starts_with("Example:") {
                if let Some(ex) = extract_code_text(sib) {
                    opt.examples.push(ex);
                } else {
                    let ex = text.strip_prefix("Example:").unwrap_or(&text).trim().to_string();
                    if !ex.is_empty() {
                        opt.examples.push(ex);
                    }
                }
            } else if text.starts_with("Declared in:") || text.starts_with("Declared by:") {
                opt.declared_by = extract_links(sib);
            } else if tag == "p" && !text.is_empty() {
                description_parts.push(text);
            } else if tag == "pre" {
                opt.examples.push(extract_text(sib));
            }
        }

        if !description_parts.is_empty() {
            opt.description = Some(description_parts.join("\n\n"));
        }

        opt.raw_html = Some(raw_parts.join("\n"));
        options.push(opt);
    }

    Ok(options)
}
