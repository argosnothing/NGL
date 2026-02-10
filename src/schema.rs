// Base schema defining the language of NGL data structure
// Defines components of an NGLRequest and an NGLResponse
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NGLRequest {
    pub search_term: Option<String>,
    // TODO: We could probably make this an enum?
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
    /// The name of the function.
    pub name: String,
    pub signature: Option<String>,
    /// This is the body of the documentation
    /// Examples will likely be nested in here
    pub content: NGLRaw,
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
    pub title: NGLRaw,
    /// An entire guide
    /// For now, and for simplicity
    /// Guide will carry with them their own formatting
    /// In this case markdown, and it's up to the caller
    /// to parse this string in a way that works for display.
    pub content: NGLRaw,
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

// Specifies format for the content
// This is because multipart documentation often has formatting
// embedded into the data, for example markdown
// as # To show headings, where's HTML will have <h1>text</h1>
// NGL's job isn't to provide sophisticated parsing of this data
// it simply needs to organize data from the sources.
// With the exception of examples, as these are strictly code blocks
// and don't need the same considerations.
// It's important to understand that it's possible for datatypes to nest other types
// this separation just makes it easier for the consumer to see the primary format of the text
// without analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NGLRaw {
    Markdown(String),
    HTML(String),
    PlainText(String),
}

impl NGLDataVariant {
    /// Returns the kind of this [`NGLDataVariant`].
    #[allow(unused)]
    pub fn kind(&self) -> NGLDataKind {
        match self {
            NGLDataVariant::Function(_) => NGLDataKind::Function,
            NGLDataVariant::Example(_) => NGLDataKind::Example,
            NGLDataVariant::Guide(_) => NGLDataKind::Guide,
            NGLDataVariant::Option(_) => NGLDataKind::Option,
            NGLDataVariant::Package(_) => todo!(),
            NGLDataVariant::Type(_) => NGLDataKind::Type,
        }
    }
}
