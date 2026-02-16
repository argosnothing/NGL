use crate::providers::{EventChannel, Provider, ProviderEvent};
use crate::schema::NGLDataKind;
use async_trait::async_trait;
use brotli2::read::BrotliDecoder;
use regex::Regex;
use sea_orm::ActiveValue::*;
use sea_orm::DbErr;
use serde::de::{self, Deserializer, MapAccess, Visitor};
use std::fmt;
use std::io::Read;
use std::sync::LazyLock;
use tokio::sync::mpsc;

pub mod schema;

static KEY_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"<Key>([^<]+)</Key>"#).unwrap());
static RELEASE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"^nixpkgs/([^/]+)/packages\.json\.br$"#).unwrap());
static CONTINUATION_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"<NextContinuationToken>([^<]+)</NextContinuationToken>"#).unwrap());
static PREFIX_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"<Prefix>(nixpkgs/[^<]+/)</Prefix>"#).unwrap());

#[derive(Default)]
pub struct NixPkgs {}

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

    async fn sync(&mut self, channel: &EventChannel, kinds: &[NGLDataKind]) -> Result<(), DbErr> {
        if !kinds.contains(&NGLDataKind::Package) {
            return Ok(());
        }

        // If you don't set that env var somewhere, you're going to have a bad time.
        let release = if let Ok(r) = std::env::var("NGL_NIXPKGS_RELEASE") {
            r
        } else {
            self.discover_release().await?
        };

        self.fetch_packages_for_release(channel, release).await
    }
}

impl NixPkgs {
    async fn discover_release(&self) -> Result<String, DbErr> {
        let mut continuation: Option<String> = None;
        let mut releases: Vec<String> = Vec::new();

        loop {
            let body = self.fetch_s3_listing(continuation.as_deref()).await?;

            for cap in KEY_RE.captures_iter(&body) {
                if let Some(release_cap) = RELEASE_RE.captures(cap.get(1).unwrap().as_str()) {
                    releases.push(release_cap.get(1).unwrap().as_str().to_string());
                }
            }

            match CONTINUATION_RE.captures(&body) {
                Some(cap) => continuation = Some(cap.get(1).unwrap().as_str().to_string()),
                None => break,
            }
        }

        if releases.is_empty() {
            let body = self.fetch_s3_listing(None).await?;
            for cap in PREFIX_RE.captures_iter(&body) {
                let prefix = cap.get(1).unwrap().as_str();
                let release = prefix
                    .trim_start_matches("nixpkgs/")
                    .trim_end_matches('/');
                releases.push(release.to_string());
            }
        }

        releases.sort();
        releases
            .pop()
            .ok_or_else(|| DbErr::Custom("failed to discover nixpkgs release".to_string()))
    }

    async fn fetch_s3_listing(&self, continuation_token: Option<&str>) -> Result<String, DbErr> {
        let mut url =
            String::from("https://nix-releases.s3.amazonaws.com/?list-type=2&prefix=nixpkgs/");
        if let Some(token) = continuation_token {
            url.push_str(&format!(
                "&continuation-token={}",
                urlencoding::encode(token)
            ));
        }

        let resp = reqwest::get(&url)
            .await
            .map_err(|e| DbErr::Custom(format!("S3 list error: {}", e)))?;

        if !resp.status().is_success() {
            return Err(DbErr::Custom(format!(
                "S3 returned status {}",
                resp.status()
            )));
        }

        resp.text()
            .await
            .map_err(|e| DbErr::Custom(e.to_string()))
    }

    async fn fetch_packages_for_release(
        &self,
        channel: &EventChannel,
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
            let (name, pkg_value) = result.map_err(DbErr::Custom)?;
            let meta = pkg_value.get("meta");

            let version = get_str(&pkg_value, "version");
            let description = meta.and_then(|m| get_str(m, "description"));
            let homepage = meta.and_then(|m| get_str_or_first(m, "homepage"));
            let license = meta.and_then(extract_license);
            let source_code_url = meta
                .and_then(|m| get_str(m, "position"))
                .map(|pos| position_to_github_url(&pos));
            let broken = meta.and_then(|m| get_bool(m, "broken")).unwrap_or(false);
            let unfree = meta.and_then(|m| get_bool(m, "unfree")).unwrap_or(false);

            channel
                .send(ProviderEvent::Package(
                    crate::db::entities::package::ActiveModel {
                        id: NotSet,
                        provider_name: Set("nixpkgs".to_string()),
                        name: Set(name),
                        version: Set(version),
                        format: Set(
                            crate::db::enums::documentation_format::DocumentationFormat::PlainText,
                        ),
                        data: Set(serde_json::to_string(&pkg_value).unwrap_or_default()),
                        description: Set(description),
                        homepage: Set(homepage),
                        license: Set(license),
                        source_code_url: Set(source_code_url),
                        broken: Set(broken),
                        unfree: Set(unfree),
                    },
                ))
                .await;
        }

        parse_handle
            .await
            .map_err(|e| DbErr::Custom(e.to_string()))?;

        Ok(())
    }
}

fn get_str(v: &serde_json::Value, key: &str) -> Option<String> {
    v.get(key).and_then(|x| x.as_str()).map(String::from)
}

fn get_bool(v: &serde_json::Value, key: &str) -> Option<bool> {
    v.get(key).and_then(|x| x.as_bool())
}

fn get_str_or_first(v: &serde_json::Value, key: &str) -> Option<String> {
    v.get(key).and_then(|x| {
        x.as_str()
            .map(String::from)
            .or_else(|| x.as_array()?.first()?.as_str().map(String::from))
    })
}

fn extract_license(meta: &serde_json::Value) -> Option<String> {
    let l = meta.get("license")?;

    if let Some(s) = l.as_str() {
        return Some(s.to_string());
    }

    if let Some(obj) = l.as_object() {
        return obj
            .get("spdxId")
            .or(obj.get("fullName"))
            .and_then(|v| v.as_str())
            .map(String::from);
    }

    if let Some(arr) = l.as_array() {
        return arr.first().and_then(|v| {
            v.as_str().map(String::from).or_else(|| {
                let obj = v.as_object()?;
                obj.get("spdxId")
                    .or(obj.get("fullName"))
                    .and_then(|x| x.as_str())
                    .map(String::from)
            })
        });
    }

    None
}

fn position_to_github_url(pos: &str) -> String {
    let (file, line) = pos.split_once(':').unwrap_or((pos, "1"));
    format!(
        "https://github.com/NixOS/nixpkgs/blob/master/{}#L{}",
        file, line
    )
}

// Streaming JSON deserializer for packages.json
// Processes one package at a time without loading the entire file into memory
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
