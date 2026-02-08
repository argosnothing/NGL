use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NGLRequest {
    pub search_term: Option<String>,
    pub providers: Option<Vec<String>>,
    pub kinds: Option<Vec<NGLDataKind>>,
}

/// The data coming back from the crate
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
    /// Code block parsed as plaintext with formatting preserved
    pub code: String,
    /// Language of the code block to give to the caller
    pub language: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuideData {
    /// The name of this guide, captured through either
    /// url route convention or by a header the provider
    /// can reliable connect with a guide's title.
    /// This is likely what i'll use initially for querying
    /// guides, at least till we get fts5 impl
    pub title: String,
    /// An entire guide
    /// For now, and for simplicity
    /// Guide will carry with them their own formatting
    /// In this case markdown, and it's up to the caller
    /// to parse this string in a way that works for display.
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
