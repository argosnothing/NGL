use crate::db::entities::option as option_entity;
use crate::db::enums::documentation_format::DocumentationFormat;
use crate::providers::{ProviderEvent, ProviderInformation, Sink};
use crate::schema::NGLDataKind;
use crate::utils::{fetch_source, html_to_markdown};
use scraper::{ElementRef, Html, Selector};
use sea_orm::ActiveValue::*;
use sea_orm::DbErr;
use std::sync::Arc;

use super::{ConfigProvider, TemplateProviderConfig};

pub struct RenderDocsProvider {
    info: ProviderInformation,
}

impl RenderDocsProvider {
    pub fn from_config(cfg: &TemplateProviderConfig) -> Self {
        Self {
            info: cfg.to_provider_info(None),
        }
    }

    async fn parse_options(&self, sink: Arc<dyn Sink>) -> Result<(), DbErr> {
        let html = fetch_source(&self.info.source)
            .await
            .map_err(|e| DbErr::Custom(format!("Failed to fetch source: {}", e)))?;

        let options = parse_renderdocs_html(&html)
            .map_err(|e| DbErr::Custom(format!("Failed to parse HTML: {}", e)))?;

        for opt in options {
            let markdown = opt
                .raw_html
                .map(|h| html_to_markdown(&h))
                .unwrap_or_default();
            sink.emit(ProviderEvent::Option(option_entity::ActiveModel {
                id: NotSet,
                provider_name: Set(self.info.name.clone()),
                name: Set(opt.name),
                type_signature: Set(opt.option_type),
                default_value: Set(opt.default),
                format: Set(DocumentationFormat::Markdown),
                data: Set(markdown),
            }))
            .await?;
        }

        Ok(())
    }
}

impl ConfigProvider for RenderDocsProvider {
    fn provider_info(&self) -> &ProviderInformation {
        &self.info
    }

    async fn sync(&mut self, sink: Arc<dyn Sink>, kinds: &[NGLDataKind]) -> Result<(), DbErr> {
        if kinds.contains(&NGLDataKind::Option) && self.info.kinds.contains(&NGLDataKind::Option) {
            self.parse_options(sink).await?;
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
    pub example: Option<String>,
    pub declared_by: Vec<String>,
    pub raw_html: Option<String>,
}

pub fn parse_renderdocs_html(html: &str) -> Result<Vec<ParsedOption>, String> {
    let document = Html::parse_document(html);

    let dl_selector =
        Selector::parse("dl.variablelist").map_err(|e| format!("Invalid selector: {:?}", e))?;
    let dt_selector = Selector::parse("dt").map_err(|e| format!("Invalid selector: {:?}", e))?;
    let dd_selector = Selector::parse("dd").map_err(|e| format!("Invalid selector: {:?}", e))?;
    let term_selector =
        Selector::parse("span.term").map_err(|e| format!("Invalid selector: {:?}", e))?;
    let option_code_selector =
        Selector::parse("code.option").map_err(|e| format!("Invalid selector: {:?}", e))?;

    let dl = document
        .select(&dl_selector)
        .next()
        .ok_or_else(|| "No dl.variablelist found in HTML".to_string())?;

    let dts: Vec<ElementRef> = dl.select(&dt_selector).collect();
    let dds: Vec<ElementRef> = dl.select(&dd_selector).collect();

    if dts.len() != dds.len() {
        return Err(format!(
            "Mismatched dt/dd counts: {} dt vs {} dd",
            dts.len(),
            dds.len()
        ));
    }

    let mut options = Vec::with_capacity(dts.len());

    for (dt, dd) in dts.iter().zip(dds.iter()) {
        let mut opt = ParsedOption::default();

        if let Some(term) = dt.select(&term_selector).next() {
            if let Some(code) = term.select(&option_code_selector).next() {
                opt.name = get_text_content(&code);
            } else {
                opt.name = get_text_content(&term);
            }
        }

        if opt.name.is_empty() {
            continue;
        }

        opt.raw_html = Some(dd.inner_html());

        let props = extract_properties(dd);
        opt.option_type = props.type_val;
        opt.default = props.default;
        opt.example = props.example;
        opt.declared_by = props.declared_by;

        opt.description = extract_description(dd);

        options.push(opt);
    }

    if options.is_empty() {
        return Err("No options found in HTML".to_string());
    }

    Ok(options)
}

struct ExtractedProperties {
    type_val: Option<String>,
    default: Option<String>,
    example: Option<String>,
    declared_by: Vec<String>,
}

fn extract_properties(dd: &ElementRef) -> ExtractedProperties {
    let p_selector = Selector::parse("p").unwrap();
    let pre_code_selector = Selector::parse("pre code").unwrap();
    let emphasis_selector = Selector::parse("span.emphasis").unwrap();
    let table_link_selector = Selector::parse("table a[href]").unwrap();

    let mut props = ExtractedProperties {
        type_val: None,
        default: None,
        example: None,
        declared_by: Vec::new(),
    };

    let p_elements: Vec<ElementRef> = dd.select(&p_selector).collect();
    let pre_codes: Vec<ElementRef> = dd.select(&pre_code_selector).collect();

    for (i, p) in p_elements.iter().enumerate() {
        if p.select(&emphasis_selector).next().is_none() {
            continue;
        }

        let p_text = get_text_content(p);

        for prop_name in ["Type:", "Default:", "Example:"] {
            if !p_text.contains(prop_name) {
                continue;
            }

            let value = if let Some(inline) = extract_inline_value(&p_text, prop_name) {
                Some(inline)
            } else {
                pre_codes
                    .get(i.saturating_sub(p_elements.len() - pre_codes.len()))
                    .or_else(|| pre_codes.first())
                    .map(|pre| normalize_value(&get_text_content(pre)))
            };

            match prop_name {
                "Type:" => props.type_val = value,
                "Default:" => props.default = value,
                "Example:" => props.example = value,
                _ => {}
            }
        }
    }

    for link in dd.select(&table_link_selector) {
        if let Some(href) = link.value().attr("href") {
            if href.starts_with("http") {
                props.declared_by.push(href.to_string());
            }
        }
    }

    props
}

fn extract_inline_value(text: &str, prop_name: &str) -> Option<String> {
    if let Some(idx) = text.find(prop_name) {
        let value = text[idx + prop_name.len()..].trim();
        if !value.is_empty() {
            return Some(normalize_value(value));
        }
    }
    None
}

fn extract_description(dd: &ElementRef) -> Option<String> {
    let p_selector = Selector::parse("p").unwrap();
    let emphasis_selector = Selector::parse("span.emphasis").unwrap();

    for p in dd.select(&p_selector) {
        if p.select(&emphasis_selector).next().is_some() {
            let text = get_text_content(&p);
            if text.contains("Type:")
                || text.contains("Default:")
                || text.contains("Example:")
                || text.contains("Declared by:")
            {
                continue;
            }
        }

        let text = get_text_content(&p).trim().to_string();
        if !text.is_empty() {
            return Some(text);
        }
    }

    None
}

fn get_text_content(el: &ElementRef) -> String {
    el.text().collect::<Vec<_>>().join("")
}

fn normalize_value(s: &str) -> String {
    let s = s.trim();

    let s = if s.len() > 1 && s.starts_with('`') && s.ends_with('`') {
        &s[1..s.len() - 1]
    } else {
        s
    };

    s.replace('"', "\"")
        .replace('"', "\"")
        .replace('\u{2018}', "'") // left single quote
        .replace('\u{2019}', "'") // right single quote
        .replace('–', "--")
        .replace('…', "..")
}
