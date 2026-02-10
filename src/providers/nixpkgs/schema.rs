use serde::{Deserialize, Deserializer};
use std::collections::HashMap;

pub type NixpkgsIndex = HashMap<String, Package>;

#[derive(Debug, Clone, Deserialize)]
pub struct Package {
    #[serde(flatten)]
    pub base: BasePackage,
    pub meta: Meta,
    pub version: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BasePackage {
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Meta {
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub long_description: String,
    #[serde(default)]
    pub main_program: String,
    #[serde(default, deserialize_with = "deserialize_elem_or_slice")]
    pub homepage: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_elem_or_slice")]
    pub license: Vec<License>,
    #[serde(default)]
    pub broken: bool,
    #[serde(default)]
    pub unfree: bool,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub position: String,
    #[serde(default)]
    pub platforms: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum License {
    String(String),
    Struct {
        free: bool,
        #[serde(rename = "fullName")]
        full_name: String,
        #[serde(rename = "spdxId")]
        spd_id: String,
    },
}

fn deserialize_elem_or_slice<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum ElemOrSlice<T> {
        Elem(T),
        Slice(Vec<T>),
    }

    match ElemOrSlice::deserialize(deserializer)? {
        ElemOrSlice::Elem(e) => Ok(vec![e]),
        ElemOrSlice::Slice(s) => Ok(s),
    }
}
