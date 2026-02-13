#![allow(unused)]
use crate::providers::Provider;
use async_trait::async_trait;
pub mod schema;

use brotli2::read::BrotliDecoder;
use sea_orm::ActiveValue::*;

pub struct NixPkgs {}

impl NixPkgs {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Provider for NixPkgs {
    fn get_info(&self) -> super::ProviderInformation {
        super::ProviderInformation {
            kinds: vec![crate::schema::NGLDataKind::Package],
            name: "nixpkgs".to_string(),
            source: "https://releases.nixos.org/nixpkgs/".to_string(),
        }
    }

    async fn fetch_functions(&mut self) -> Vec<crate::db::entities::function::ActiveModel> {
        vec![]
    }

    async fn fetch_examples(&mut self) -> Vec<crate::db::example::ActiveModel> {
        vec![]
    }

    async fn fetch_guides(&mut self) -> Vec<crate::db::entities::guide::ActiveModel> {
        vec![]
    }

    async fn fetch_options(&mut self) -> Vec<crate::db::entities::option::ActiveModel> {
        vec![]
    }

    async fn fetch_packages(&mut self) -> Vec<crate::db::entities::package::ActiveModel> {
        if let Ok(release) = std::env::var("NGL_NIXPKGS_RELEASE") {
            return self.fetch_packages_for_release(release).await;
        }
        {
            let mut continuation: Option<String> = None;
            let mut found: Vec<String> = Vec::new();

            loop {
                let mut url = String::from("https://nix-releases.s3.amazonaws.com/?list-type=2&prefix=nixpkgs/");
                if let Some(token) = &continuation {
                    let enc = urlencoding::encode(token);
                    url.push_str(&format!("&continuation-token={}", enc));
                }

                let resp = match reqwest::get(&url).await {
                    Ok(r) if r.status().is_success() => r,
                    Ok(r) => {
                        eprintln!("unexpected status {} when listing S3 via HTTP", r.status());
                        break;
                    }
                    Err(e) => {
                        eprintln!("http s3 list error: {}", e);
                        break;
                    }
                };

                let body = match resp.text().await {
                    Ok(t) => t,
                    Err(e) => {
                        eprintln!("failed to read s3 list body: {}", e);
                        break;
                    }
                };

                let key_re = regex::Regex::new(r#"<Key>([^<]+)</Key>"#).unwrap();
                for cap in key_re.captures_iter(&body) {
                    let key = cap.get(1).unwrap().as_str();
                    if key.ends_with("/packages.json.br") {
                        if let Some(cap2) = regex::Regex::new(r#"^nixpkgs/([^/]+)/packages.json.br$"#).unwrap().captures(key) {
                            if let Some(m) = cap2.get(1) {
                                found.push(m.as_str().to_string());
                            }
                        }
                    }
                }

                let next_re = regex::Regex::new(r#"<NextContinuationToken>([^<]+)</NextContinuationToken>"#).unwrap();
                if let Some(cap) = next_re.captures(&body) {
                    continuation = Some(cap.get(1).unwrap().as_str().to_string());
                } else {
                    break;
                }
            }

            if !found.is_empty() {
                found.sort();
                let release = found.pop().unwrap();
                println!("nixpkgs: selected release (from S3-http) {}", release);
                return self.fetch_packages_for_release(release).await;
            } else {
                eprintln!("HTTP S3 ListObjectsV2 did not find packages.json.br; falling back to XML index");
            }
        }

        let s3_index = "https://nix-releases.s3.amazonaws.com/?prefix=nixpkgs/&delimiter=/";
        let body = match reqwest::get(s3_index).await {
            Ok(r) if r.status().is_success() => match r.text().await {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("failed to read S3 index body: {}", e);
                    return vec![];
                }
            },
            Ok(r) => {
                eprintln!("unexpected status {} fetching {}", r.status(), s3_index);
                return vec![];
            }
            Err(e) => {
                eprintln!("failed to fetch S3 index: {}", e);
                return vec![];
            }
        };
        let key_re = regex::Regex::new(r#"<Key>(nixpkgs/([^<]+?)/packages.json.br)</Key>"#).unwrap();
        let mut candidate_releases: Vec<String> = key_re
            .captures_iter(&body)
            .filter_map(|cap| cap.get(2).map(|m| m.as_str().to_string()))
            .collect();

        if !candidate_releases.is_empty() {
            candidate_releases.sort();
            let release = candidate_releases.pop().unwrap();
            println!("nixpkgs: selected release (from packages.json.br keys) {}", release);
            return self.fetch_packages_for_release(release).await;
        }

        let re = regex::Regex::new(r#"<Prefix>(nixpkgs/[^<]+/)</Prefix>"#).unwrap();
        let mut prefixes: Vec<String> = re
            .captures_iter(&body)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
            .collect();

        if prefixes.is_empty() {
            eprintln!("failed to discover nixpkgs release prefixes from S3 index");
            return vec![];
        }

        let mut releases: Vec<String> = prefixes
            .into_iter()
            .map(|p| p.trim_end_matches('/').trim_start_matches("nixpkgs/").to_string())
            .collect();

        releases.sort();
        let release = releases.pop().unwrap();
        println!("nixpkgs: selected release (from prefixes) {}", release);

        return self.fetch_packages_for_release(release).await;
    }

    async fn fetch_types(&mut self) -> Vec<crate::db::entities::r#type::ActiveModel> {
        vec![]
    }
}

impl NixPkgs {
    async fn fetch_packages_for_release(
        &mut self,
        release: String,
    ) -> Vec<crate::db::entities::package::ActiveModel> {
        let rel = release.trim_start_matches("nixpkgs/");
        let url = format!(
            "https://releases.nixos.org/nixpkgs/{}/packages.json.br",
            rel
        );

        let resp = match reqwest::get(&url).await {
            Ok(r) if r.status().is_success() => r,
            Ok(r) => {
                eprintln!("unexpected http status {} when fetching {}", r.status(), url);
                return vec![];
            }
            Err(e) => {
                eprintln!("request error fetching nixpkgs packages: {}", e);
                return vec![];
            }
        };

        let bytes = match resp.bytes().await {
            Ok(b) => b,
            Err(e) => {
                eprintln!("failed to read response bytes: {}", e);
                return vec![];
            }
        };

        eprintln!("downloaded {} bytes for {}", bytes.len(), url);
        let prefix_len = std::cmp::min(16, bytes.len());
        if prefix_len > 0 {
            let mut s = String::new();
            for b in &bytes[..prefix_len] {
                s.push_str(&format!("{:02x}", b));
            }
            eprintln!("first {} bytes (hex): {}", prefix_len, s);
        }

        let decompressed = if bytes.first().map(|b| *b) == Some(b'{') {
            match String::from_utf8(bytes.to_vec()) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("utf8 decode error: {}", e);
                    return vec![];
                }
            }
        } else {
            let cursor = std::io::Cursor::new(bytes);
            let mut decoder = BrotliDecoder::new(cursor);

            let mut s = String::new();
            if let Err(e) = std::io::Read::read_to_string(&mut decoder, &mut s) {
                eprintln!("brotli decompress error: {}", e);
                return vec![];
            }
            s
        };

        let json: serde_json::Value = match serde_json::from_str(&decompressed) {
            Ok(j) => j,
            Err(e) => {
                eprintln!("failed to parse packages.json: {}", e);
                return vec![];
            }
        };

        let packages = match json.get("packages") {
            Some(p) if p.is_object() => p.as_object().unwrap(),
            _ => {
                eprintln!("packages.json missing 'packages' object");
                return vec![];
            }
        };

        let mut out = Vec::new();
        for (name, pkg_value) in packages.iter() {
            let version = pkg_value.get("version").and_then(|v| v.as_str()).map(|s| s.to_string());

            let data = match serde_json::to_string(pkg_value) {
                Ok(s) => s,
                Err(_) => continue,
            };

            let model = crate::db::entities::package::ActiveModel {
                id: NotSet,
                provider_name: Set("nixpkgs".to_string()),
                name: Set(name.clone()),
                version: Set(version),
                format: Set(crate::db::enums::documentation_format::DocumentationFormat::PlainText),
                data: Set(data),
            };

            out.push(model);
        }

        out
    }
}
