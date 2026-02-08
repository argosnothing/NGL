use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NGLRequest {
    pub search_term: Option<String>,
    pub providers: Option<Vec<String>>,
    pub kinds: Option<Vec<NGLDataKind>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NGLResponse {
    pub provider_name: String,
    pub matches: Vec<NGLData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NGLData {
    pub kind: NGLDataKind,
    pub data: NGLDataVariant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NGLDataVariant {
    Function(FunctionData),
    Example(ExampleData),
    Guide(GuideData),
    Option(OptionData),
    Package(PackageData),
    Type(TypeData),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionData {
    pub name: String,
    pub signature: Option<String>,
    pub description: Option<String>,
    pub examples: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExampleData {
    pub code: String,
    pub language: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuideData {
    pub title: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionData {
    pub name: String,
    pub option_type: Option<String>,
    pub default_value: Option<String>,
    pub description: Option<String>,
    pub example: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageData {
    pub name: String,
    pub version: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeData {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NGLDataKind {
    Function,
    Example,
    Guide,
    Option,
    Package,
    Type,
}
