#![allow(unused)]
use async_trait::async_trait;
use scraper::{ElementRef, Html, Selector};
use sea_orm::{ActiveValue::*, DbErr};

use crate::{
    NGLDataKind,
    db::{
        entities::{example, guide},
        enums::documentation_format::DocumentationFormat,
    },
    providers::{EventChannel, LinkedExample, Provider, ProviderEvent, ProviderInformation},
    utils::{extract_examples_html, fetch_source},
};

static URL: &str = "https://nixos.org/manual/nixos/stable/";
static PROVIDER_NAME: &str = "nixos_manual";

#[derive(Default)]
pub struct NixosManual {}

#[derive(Debug, Clone)]
struct ParsedGuide {
    id: String,
    title: String,
    content_html: String,
    link: String,
    parent_id: Option<String>,
    depth: usize,
}

#[async_trait]
impl Provider for NixosManual {
    async fn sync(&mut self, channel: &EventChannel, kinds: &[NGLDataKind]) -> Result<(), DbErr> {
        let include_guides = kinds.contains(&NGLDataKind::Guide);
        let include_examples = kinds.contains(&NGLDataKind::Example);

        let html = fetch_source(URL)
            .await
            .map_err(|e| DbErr::Custom(format!("Failed to fetch NixOS manual: {}", e)))?;

        let guides = parse_manual(&html);

        let mut id_to_temp: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();

        for guide in &guides {
            id_to_temp.insert(guide.id.clone(), guide.id.clone());

            let (processed_content, extracted) = if include_examples {
                extract_examples_html(&guide.content_html)
            } else {
                (guide.content_html.clone(), vec![])
            };

            let linked_examples: Vec<LinkedExample> = extracted
                .into_iter()
                .map(|ex| LinkedExample {
                    placeholder_key: ex.placeholder_key,
                    model: example::ActiveModel {
                        id: NotSet,
                        provider_name: Set(PROVIDER_NAME.to_owned()),
                        language: Set(ex.language),
                        data: Set(ex.data),
                        source_kind: Set(Some(format!("{:?}", NGLDataKind::Guide))),
                        source_link: Set(Some(guide.link.clone())),
                    },
                })
                .collect();

            // Only emit based on what kinds are requested
            match (include_guides, include_examples) {
                (true, true) if !linked_examples.is_empty() => {
                    // Emit guide with linked examples
                    channel
                        .send(ProviderEvent::GuideWithExamples(
                            guide::ActiveModel {
                                id: NotSet,
                                provider_name: Set(PROVIDER_NAME.to_owned()),
                                title: Set(guide.title.clone()),
                                format: Set(DocumentationFormat::HTML),
                                data: Set(processed_content),
                                link: Set(guide.link.clone()),
                            },
                            linked_examples,
                        ))
                        .await;
                }
                (true, _) => {
                    channel
                        .send(ProviderEvent::Guide(guide::ActiveModel {
                            id: NotSet,
                            provider_name: Set(PROVIDER_NAME.to_owned()),
                            title: Set(guide.title.clone()),
                            format: Set(DocumentationFormat::HTML),
                            data: Set(processed_content),
                            link: Set(guide.link.clone()),
                        }))
                        .await;
                }
                (false, true) => {
                    for ex in linked_examples {
                        channel.send(ProviderEvent::Example(ex.model)).await;
                    }
                }
                (false, false) => unreachable!(), // Already handled above
            }
        }

        if include_guides {
            for guide in &guides {
                if let Some(ref parent_id) = guide.parent_id {
                    let parent_link = format!("{}#{}", URL, parent_id);
                    channel
                        .send(ProviderEvent::GuideXref(parent_link, guide.link.clone()))
                        .await;
                }
            }
        }

        Ok(())
    }

    fn get_info(&self) -> ProviderInformation {
        ProviderInformation {
            name: PROVIDER_NAME.to_string(),
            source: URL.to_string(),
            kinds: vec![NGLDataKind::Guide, NGLDataKind::Example],
            sync_interval_hours: Some(168), // weekly
        }
    }
}

fn parse_manual(html: &str) -> Vec<ParsedGuide> {
    let document = Html::parse_document(html);
    let mut guides = Vec::new();
    let mut seen_ids: std::collections::HashSet<String> = std::collections::HashSet::new();

    // Select all structural elements we care about
    let selector = Selector::parse("div.preface, div.part, div.chapter, div.section").unwrap();

    for element in document.select(&selector) {
        if let Some(guide) = parse_guide_element(&element) {
            if seen_ids.insert(guide.id.clone()) {
                guides.push(guide);
            }
        }
    }

    guides
}

fn parse_guide_element(element: &ElementRef) -> Option<ParsedGuide> {
    let (id, title) = extract_title_and_id(element)?;

    let content_html = extract_own_content(element);

    let link = format!("{}#{}", URL, id);

    let class = element.value().attr("class").unwrap_or("");
    let depth = if class.contains("preface") || class.contains("part") {
        0
    } else if class.contains("chapter") {
        1
    } else {
        2 // section
    };

    let parent_id = find_parent_id(element);

    Some(ParsedGuide {
        id,
        title,
        content_html,
        link,
        parent_id,
        depth,
    })
}

fn extract_own_content(element: &ElementRef) -> String {
    let mut content = String::new();

    for child in element.children() {
        // Skip nested structural elements (they'll be parsed as their own guides)
        if let Some(el) = child.value().as_element() {
            let tag = el.name();
            let class = el.attr("class").unwrap_or("");

            // Stop at nested sections/chapters - these are separate guides
            if tag == "div"
                && (class.contains("section")
                    || class.contains("chapter")
                    || class.contains("part"))
            {
                continue;
            }
        }

        // Render this child node to HTML
        if let Some(el_ref) = ElementRef::wrap(child) {
            content.push_str(&el_ref.html());
        } else if let Some(text) = child.value().as_text() {
            content.push_str(text);
        }
    }

    content
}

fn extract_title_and_id(element: &ElementRef) -> Option<(String, String)> {
    for tag in ["h1", "h2", "h3", "h4"] {
        if let Ok(sel) = Selector::parse(tag) {
            if let Some(heading) = element.select(&sel).next() {
                let id = heading.value().id()?.to_string();
                let text: String = heading.text().collect::<Vec<_>>().join("");
                let cleaned = text.trim().to_string();
                if !cleaned.is_empty() {
                    return Some((id, cleaned));
                }
            }
        }
    }
    None
}

fn find_parent_id(element: &ElementRef) -> Option<String> {
    let mut current = element.parent();
    while let Some(node) = current {
        if let Some(el) = node.value().as_element() {
            if let Some(class) = el.attr("class") {
                if class.contains("part") || class.contains("chapter") || class.contains("section")
                {
                    if let Some(el_ref) = ElementRef::wrap(node) {
                        for tag in ["h1", "h2", "h3", "h4"] {
                            if let Ok(sel) = Selector::parse(tag) {
                                if let Some(heading) = el_ref.select(&sel).next() {
                                    if let Some(id) = heading.value().id() {
                                        return Some(id.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        current = node.parent();
    }
    None
}
