use crate::db::enums::language::Language;
use regex::Regex;

pub struct ExtractedExample {
    pub placeholder_key: String,
    pub language: Option<Language>,
    pub data: String,
}

pub fn extract_examples_html(content: &str) -> (String, Vec<ExtractedExample>) {
    let re =
        Regex::new(r#"<pre[^>]*>\s*<code[^>]*class="([^"]*)"[^>]*>([\s\S]*?)</code>\s*</pre>"#)
            .unwrap();
    let mut examples = Vec::new();
    let mut counter = 0;

    let result = re.replace_all(content, |caps: &regex::Captures| {
        let class_str = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let code = caps.get(2).map(|m| m.as_str()).unwrap_or("");
        let key = format!("ex{}", counter);
        counter += 1;

        let language = parse_language_from_class(class_str);

        examples.push(ExtractedExample {
            placeholder_key: key.clone(),
            language,
            data: html_escape::decode_html_entities(code).to_string(),
        });

        format!("{{{{NGL_EX:{}}}}}", key)
    });

    (result.to_string(), examples)
}

fn parse_language_from_class(class_str: &str) -> Option<Language> {
    let lower = class_str.to_lowercase();
    if let Some(rest) = lower.strip_prefix("language-") {
        return match rest.split_whitespace().next() {
            Some("nix") => Some(Language::Nix),
            _ => None,
        };
    }
    for word in lower.split_whitespace() {
        if word == "nix" {
            return Some(Language::Nix);
        }
    }
    None
}

pub fn extract_examples_markdown(content: &str) -> (String, Vec<ExtractedExample>) {
    let re = Regex::new(r"```(\w*)\n([\s\S]*?)```").unwrap();
    let mut examples = Vec::new();
    let mut counter = 0;

    let result = re.replace_all(content, |caps: &regex::Captures| {
        let lang_str = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let code = caps.get(2).map(|m| m.as_str()).unwrap_or("");
        let key = format!("ex{}", counter);
        counter += 1;

        let language = match lang_str.to_lowercase().as_str() {
            "nix" => Some(Language::Nix),
            _ => None,
        };

        examples.push(ExtractedExample {
            placeholder_key: key.clone(),
            language,
            data: code.to_string(),
        });

        format!("{{{{NGL_EX:{}}}}}", key)
    });

    (result.to_string(), examples)
}

pub fn stitch_examples(content: &str, examples: &[(String, String)]) -> String {
    let mut result = content.to_string();
    for (key, data) in examples {
        let placeholder = format!("{{{{NGL_EX:{}}}}}", key);
        result = result.replace(&placeholder, data);
    }
    result
}

pub fn strip_placeholders(content: &str) -> String {
    static PLACEHOLDER_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let re = PLACEHOLDER_RE.get_or_init(|| regex::Regex::new(r"\{\{NGL_EX:[^}]+\}\}").unwrap());
    re.replace_all(content, "").to_string()
}

pub fn html_to_markdown(html: &str) -> String {
    html2md::parse_html(html)
}

pub fn is_url(source: &str) -> bool {
    source.starts_with("http://") || source.starts_with("https://")
}

pub async fn fetch_source(
    source: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    if is_url(source) {
        let client = reqwest::Client::builder()
            .user_agent("NGL/0.1 (Nix Global Lookup)")
            .timeout(std::time::Duration::from_secs(60))
            .build()?;

        let mut last_error = None;
        for attempt in 0..3 {
            if attempt > 0 {
                tokio::time::sleep(std::time::Duration::from_secs(2u64.pow(attempt))).await;
            }
            match client.get(source).send().await {
                Ok(resp) => {
                    if !resp.status().is_success() {
                        return Err(format!("HTTP error: {}", resp.status()).into());
                    }
                    return Ok(resp.text().await?);
                }
                Err(e) => {
                    last_error = Some(e);
                }
            }
        }
        Err(last_error.unwrap().into())
    } else {
        Ok(tokio::fs::read_to_string(source).await?)
    }
}
