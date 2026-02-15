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
        let resp = reqwest::get(source).await?;
        if !resp.status().is_success() {
            return Err(format!("HTTP error: {}", resp.status()).into());
        }
        Ok(resp.text().await?)
    } else {
        Ok(tokio::fs::read_to_string(source).await?)
    }
}
