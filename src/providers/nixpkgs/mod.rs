#![allow(unused)]
use crate::providers::{Provider, ProviderEvent, Sink};
use crate::schema::NGLDataKind;
use async_trait::async_trait;
use sea_orm::DbErr;
use std::sync::Arc;
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
            kinds: vec![NGLDataKind::Package],
            name: "nixpkgs".to_string(),
            source: "https://releases.nixos.org/nixpkgs/".to_string(),
        }
    }

    async fn sync(&mut self, sink: Arc<dyn Sink>, kinds: &[NGLDataKind]) -> Result<(), DbErr> {
        if !kinds.contains(&NGLDataKind::Package) {
            return Ok(());
        }

        let release = if let Ok(r) = std::env::var("NGL_NIXPKGS_RELEASE") {
            r
        } else {
            self.discover_release().await?
        };

        self.fetch_packages_for_release(sink, release).await
    }
}

impl NixPkgs {
    async fn discover_release(&self) -> Result<String, DbErr> {
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
                    return Err(DbErr::Custom(format!("unexpected status {} when listing S3", r.status())));
                }
                Err(e) => {
                    return Err(DbErr::Custom(format!("http s3 list error: {}", e)));
                }
            };

            let body = resp.text().await.map_err(|e| DbErr::Custom(e.to_string()))?;

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
            return Ok(found.pop().unwrap());
        }

        let s3_index = "https://nix-releases.s3.amazonaws.com/?prefix=nixpkgs/&delimiter=/";
        let body = reqwest::get(s3_index)
            .await
            .map_err(|e| DbErr::Custom(e.to_string()))?
            .text()
            .await
            .map_err(|e| DbErr::Custom(e.to_string()))?;

        let key_re = regex::Regex::new(r#"<Key>(nixpkgs/([^<]+?)/packages.json.br)</Key>"#).unwrap();
        let mut candidate_releases: Vec<String> = key_re
            .captures_iter(&body)
            .filter_map(|cap| cap.get(2).map(|m| m.as_str().to_string()))
            .collect();

        if !candidate_releases.is_empty() {
            candidate_releases.sort();
            return Ok(candidate_releases.pop().unwrap());
        }

        let re = regex::Regex::new(r#"<Prefix>(nixpkgs/[^<]+/)</Prefix>"#).unwrap();
        let mut releases: Vec<String> = re
            .captures_iter(&body)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().trim_end_matches('/').trim_start_matches("nixpkgs/").to_string()))
            .collect();

        if releases.is_empty() {
            return Err(DbErr::Custom("failed to discover nixpkgs release".to_string()));
        }

        releases.sort();
        Ok(releases.pop().unwrap())
    }

    async fn fetch_packages_for_release(
        &self,
        sink: Arc<dyn Sink>,
        release: String,
    ) -> Result<(), DbErr> {
        let rel = release.trim_start_matches("nixpkgs/");
        let url = format!(
            "https://releases.nixos.org/nixpkgs/{}/packages.json.br",
            rel
        );

        let resp = reqwest::get(&url)
            .await
            .map_err(|e| DbErr::Custom(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(DbErr::Custom(format!("unexpected http status {}", resp.status())));
        }

        let bytes = resp.bytes().await.map_err(|e| DbErr::Custom(e.to_string()))?;

        let decompressed = if bytes.first().map(|b| *b) == Some(b'{') {
            String::from_utf8(bytes.to_vec()).map_err(|e| DbErr::Custom(e.to_string()))?
        } else {
            let cursor = std::io::Cursor::new(bytes);
            let mut decoder = BrotliDecoder::new(cursor);
            let mut s = String::new();
            std::io::Read::read_to_string(&mut decoder, &mut s)
                .map_err(|e| DbErr::Custom(e.to_string()))?;
            s
        };

        let json: serde_json::Value =
            serde_json::from_str(&decompressed).map_err(|e| DbErr::Custom(e.to_string()))?;

        let packages = json
            .get("packages")
            .and_then(|p| p.as_object())
            .ok_or_else(|| DbErr::Custom("packages.json missing 'packages' object".to_string()))?;

        for (name, pkg_value) in packages.iter() {
            let version = pkg_value
                .get("version")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let meta = pkg_value.get("meta");

            let description = meta
                .and_then(|m| m.get("description"))
                .and_then(|d| d.as_str())
                .map(|s| s.to_string());

            let homepage = meta
                .and_then(|m| m.get("homepage"))
                .and_then(|h| {
                    if let Some(s) = h.as_str() {
                        Some(s.to_string())
                    } else if let Some(arr) = h.as_array() {
                        arr.first().and_then(|v| v.as_str()).map(|s| s.to_string())
                    } else {
                        None
                    }
                });

            let license = meta
                .and_then(|m| m.get("license"))
                .and_then(|l| {
                    if let Some(s) = l.as_str() {
                        Some(s.to_string())
                    } else if let Some(obj) = l.as_object() {
                        obj.get("spdxId").or(obj.get("fullName")).and_then(|v| v.as_str()).map(|s| s.to_string())
                    } else if let Some(arr) = l.as_array() {
                        arr.first().and_then(|v| {
                            if let Some(s) = v.as_str() {
                                Some(s.to_string())
                            } else if let Some(obj) = v.as_object() {
                                obj.get("spdxId").or(obj.get("fullName")).and_then(|x| x.as_str()).map(|s| s.to_string())
                            } else {
                                None
                            }
                        })
                    } else {
                        None
                    }
                });

            let position = meta
                .and_then(|m| m.get("position"))
                .and_then(|p| p.as_str());

            let source_code_url = position.map(|pos| {
                let parts: Vec<&str> = pos.splitn(2, ':').collect();
                let file = parts.first().unwrap_or(&"");
                let line = parts.get(1).unwrap_or(&"1");
                format!("https://github.com/NixOS/nixpkgs/blob/master/{}#L{}", file, line)
            });

            let broken = meta
                .and_then(|m| m.get("broken"))
                .and_then(|b| b.as_bool())
                .unwrap_or(false);

            let unfree = meta
                .and_then(|m| m.get("unfree"))
                .and_then(|u| u.as_bool())
                .unwrap_or(false);

            let data = serde_json::to_string(pkg_value).unwrap_or_default();

            sink.emit(ProviderEvent::Package(crate::db::entities::package::ActiveModel {
                id: NotSet,
                provider_name: Set("nixpkgs".to_string()),
                name: Set(name.clone()),
                version: Set(version),
                format: Set(crate::db::enums::documentation_format::DocumentationFormat::PlainText),
                data: Set(data),
                description: Set(description),
                homepage: Set(homepage),
                license: Set(license),
                source_code_url: Set(source_code_url),
                broken: Set(broken),
                unfree: Set(unfree),
            }))
            .await?;
        }

        Ok(())
    }
}
