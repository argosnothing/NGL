#![allow(unused)]
use crate::providers::{Provider, ProviderEvent, Sink};
use crate::schema::NGLDataKind;
use async_trait::async_trait;
use brotli2::read::BrotliDecoder;
use sea_orm::ActiveValue::*;
use sea_orm::DbErr;
use serde::Deserialize;
use serde::de::{self, Deserializer, MapAccess, Visitor};
use std::fmt;
use std::io::Read;
use tokio::sync::mpsc;

pub mod schema;

#[derive(Default)]
pub struct NixPkgs {}

/// I had to do a lot of cursed things to prevent all the incoming data flooding ram, but it works
/// pretty well considering how much data im having to work with, and it's not all happening
/// in memory so that's what is important.
impl NixPkgs {}

#[async_trait]
impl Provider for NixPkgs {
    fn get_info(&self) -> super::ProviderInformation {
        super::ProviderInformation {
            kinds: vec![NGLDataKind::Package],
            name: "nixpkgs".to_string(),
            source: "https://releases.nixos.org/nixpkgs/".to_string(),
            sync_interval_hours: Some(24),
        }
    }

    async fn sync(&mut self, sink: &dyn Sink, kinds: &[NGLDataKind]) -> Result<(), DbErr> {
        if !kinds.contains(&NGLDataKind::Package) {
            return Ok(());
        }

        /// If you don't set that env var somewhere, you're going to have a bad time.
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
            let mut url =
                String::from("https://nix-releases.s3.amazonaws.com/?list-type=2&prefix=nixpkgs/");
            if let Some(token) = &continuation {
                let enc = urlencoding::encode(token);
                url.push_str(&format!("&continuation-token={}", enc));
            }

            let resp = match reqwest::get(&url).await {
                Ok(r) if r.status().is_success() => r,
                Ok(r) => {
                    return Err(DbErr::Custom(format!(
                        "unexpected status {} when listing S3",
                        r.status()
                    )));
                }
                Err(e) => {
                    return Err(DbErr::Custom(format!("http s3 list error: {}", e)));
                }
            };

            let body = resp
                .text()
                .await
                .map_err(|e| DbErr::Custom(e.to_string()))?;

            let key_re = regex::Regex::new(r#"<Key>([^<]+)</Key>"#).unwrap();
            for cap in key_re.captures_iter(&body) {
                let key = cap.get(1).unwrap().as_str();
                if key.ends_with("/packages.json.br") {
                    if let Some(cap2) = regex::Regex::new(r#"^nixpkgs/([^/]+)/packages.json.br$"#)
                        .unwrap()
                        .captures(key)
                    {
                        if let Some(m) = cap2.get(1) {
                            found.push(m.as_str().to_string());
                        }
                    }
                }
            }

            let next_re =
                regex::Regex::new(r#"<NextContinuationToken>([^<]+)</NextContinuationToken>"#)
                    .unwrap();
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

        let key_re =
            regex::Regex::new(r#"<Key>(nixpkgs/([^<]+?)/packages.json.br)</Key>"#).unwrap();
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
            .filter_map(|cap| {
                cap.get(1).map(|m| {
                    m.as_str()
                        .trim_end_matches('/')
                        .trim_start_matches("nixpkgs/")
                        .to_string()
                })
            })
            .collect();

        if releases.is_empty() {
            return Err(DbErr::Custom(
                "failed to discover nixpkgs release".to_string(),
            ));
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
            return Err(DbErr::Custom(format!(
                "unexpected http status {}",
                resp.status()
            )));
        }

        let bytes = resp
            .bytes()
            .await
            .map_err(|e| DbErr::Custom(e.to_string()))?;

        let (tx, mut rx) = mpsc::channel::<Result<(String, serde_json::Value), String>>(64);

        // TL;DR: We got the packages.json.br file, but we don't want to load the whole thing into memory and
        // parse it with serde_json, because it's huge. Instead, we spawn a blocking task to
        // stream-parse it with serde_json's Deserializer, sending each package through a channel as it's parsed.
        // This way we can start processing packages immediately without waiting for the entire file to be parsed,
        // and we never have more than one decompressed package in memory at a time.
        let parse_handle = tokio::task::spawn_blocking(move || {
            let reader: Box<dyn Read + Send> = if bytes.first().map(|b| *b) == Some(b'{') {
                Box::new(std::io::Cursor::new(bytes))
            } else {
                let cursor = std::io::Cursor::new(bytes);
                Box::new(BrotliDecoder::new(cursor))
            };

            if let Err(e) = stream_packages_to_channel(reader, tx) {
                eprintln!("JSON parse error: {}", e);
            }
        });

        while let Some(result) = rx.recv().await {
            let (name, pkg_value) = result.map_err(|e| DbErr::Custom(e))?;

            let meta = pkg_value.get("meta");

            let version = pkg_value
                .get("version")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let description = meta
                .and_then(|m| m.get("description"))
                .and_then(|d| d.as_str())
                .map(|s| s.to_string());

            let homepage = meta.and_then(|m| m.get("homepage")).and_then(|h| {
                if let Some(s) = h.as_str() {
                    Some(s.to_string())
                } else if let Some(arr) = h.as_array() {
                    arr.first().and_then(|v| v.as_str()).map(|s| s.to_string())
                } else {
                    None
                }
            });

            let license = meta.and_then(|m| m.get("license")).and_then(|l| {
                if let Some(s) = l.as_str() {
                    Some(s.to_string())
                } else if let Some(obj) = l.as_object() {
                    obj.get("spdxId")
                        .or(obj.get("fullName"))
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                } else if let Some(arr) = l.as_array() {
                    arr.first().and_then(|v| {
                        if let Some(s) = v.as_str() {
                            Some(s.to_string())
                        } else if let Some(obj) = v.as_object() {
                            obj.get("spdxId")
                                .or(obj.get("fullName"))
                                .and_then(|x| x.as_str())
                                .map(|s| s.to_string())
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
                format!(
                    "https://github.com/NixOS/nixpkgs/blob/master/{}#L{}",
                    file, line
                )
            });

            let broken = meta
                .and_then(|m| m.get("broken"))
                .and_then(|b| b.as_bool())
                .unwrap_or(false);

            let unfree = meta
                .and_then(|m| m.get("unfree"))
                .and_then(|u| u.as_bool())
                .unwrap_or(false);

            let data = serde_json::to_string(&pkg_value).unwrap_or_default();

            sink.emit(ProviderEvent::Package(
                crate::db::entities::package::ActiveModel {
                    id: NotSet,
                    provider_name: Set("nixpkgs".to_string()),
                    name: Set(name),
                    version: Set(version),
                    format: Set(
                        crate::db::enums::documentation_format::DocumentationFormat::PlainText,
                    ),
                    data: Set(data),
                    description: Set(description),
                    homepage: Set(homepage),
                    license: Set(license),
                    source_code_url: Set(source_code_url),
                    broken: Set(broken),
                    unfree: Set(unfree),
                },
            ))
            .await?;
        }

        parse_handle
            .await
            .map_err(|e| DbErr::Custom(e.to_string()))?;

        Ok(())
    }
}

// Potentially something I should move to utils.json and generify with kinds param
// Warning: nonsense rust appeasement code ahead, make sense of it at own risk of insanity
fn stream_packages_to_channel<R: Read>(
    reader: R,
    tx: mpsc::Sender<Result<(String, serde_json::Value), String>>,
) -> Result<(), serde_json::Error> {
    struct StreamingVisitor {
        tx: mpsc::Sender<Result<(String, serde_json::Value), String>>,
    }

    impl<'de> Visitor<'de> for StreamingVisitor {
        type Value = ();

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a packages.json object with a 'packages' field")
        }

        fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            while let Some(key) = map.next_key::<String>()? {
                if key == "packages" {
                    map.next_value_seed(PackagesStreamSeed {
                        tx: self.tx.clone(),
                    })?;
                } else {
                    map.next_value::<serde::de::IgnoredAny>()?;
                }
            }
            Ok(())
        }
    }

    struct PackagesStreamSeed {
        tx: mpsc::Sender<Result<(String, serde_json::Value), String>>,
    }

    impl<'de> de::DeserializeSeed<'de> for PackagesStreamSeed {
        type Value = ();

        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_map(PackagesStreamVisitor { tx: self.tx })
        }
    }

    struct PackagesStreamVisitor {
        tx: mpsc::Sender<Result<(String, serde_json::Value), String>>,
    }

    impl<'de> Visitor<'de> for PackagesStreamVisitor {
        type Value = ();

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a map of package names to package objects")
        }

        fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            while let Some((name, value)) = map.next_entry::<String, serde_json::Value>()? {
                if self.tx.blocking_send(Ok((name, value))).is_err() {
                    break;
                }
            }
            Ok(())
        }
    }

    let mut de = serde_json::Deserializer::from_reader(reader);
    de.deserialize_map(StreamingVisitor { tx })
}
