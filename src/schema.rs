use sea_orm::{DeriveActiveEnum, EnumIter};
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
    /// URL to the documentation page (e.g., noogle.dev page)
    pub source_url: Option<String>,
    /// URL to the source code with line position
    pub source_code_url: Option<String>,
    /// Alternative names for this function (JSON array as string)
    pub aliases: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuideRef {
    pub id: i32,
    pub link: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExampleData {
    /// Code block parsed as plaintext with formatting preserved
    pub code: String,
    /// Language of the code block to give to the caller
    pub language: Option<String>,
    /// Link to parent documenation
    pub source_link: Option<String>,
    /// data kind
    pub source_kind: Option<NGLDataKind>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuideData {
    pub parent_guide: Option<GuideRef>,
    /// Sub-guides that are children of this guide
    pub sub_guides: Vec<GuideRef>,
    /// A link to the guide, or section if this is a subguide.
    pub link: String,
    /// The name of this guide, captured through either
    /// url route convention or by a header the provider
    /// can reliable connect with a guide's title.
    /// This is likely what i'll use initially for querying
    /// guides, at least till we get fts5 impl
    pub title: NGLRaw,
    /// Body of the guide.
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
    pub homepage: Option<String>,
    pub license: Option<String>,
    pub source_code_url: Option<String>,
    pub broken: bool,
    pub unfree: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeData {
    pub name: String,
    pub description: Option<String>,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, DeriveActiveEnum, EnumIter,
)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
pub enum NGLDataKind {
    #[sea_orm(num_value = 0)]
    Function,
    #[sea_orm(num_value = 1)]
    Example,
    #[sea_orm(num_value = 2)]
    Guide,
    #[sea_orm(num_value = 3)]
    Option,
    #[sea_orm(num_value = 4)]
    Package,
    #[sea_orm(num_value = 5)]
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
            NGLDataVariant::Package(_) => NGLDataKind::Package,
            NGLDataVariant::Type(_) => NGLDataKind::Type,
        }
    }
}
