/// Requests can be open ended
/// It's possible that a fzf app
/// wants to just show everything
pub struct NGLRequest {
    pub search_term: Option<String>,
    pub providers: Option<Vec<String>>,
    pub kinds: Option<Vec<NGLDataKind>>,
}

pub struct NGLResponse {
    pub provider_name: String,
    pub matches: Vec<NGLData>,
}

pub struct NGLData {
    pub kind: NGLDataKind,
    /// For now lets just consider the data a string.
    pub data: String,
}

/// It's important that we categorize what the data is representing
/// Say you have some function and you want an example of it, an
/// example is going to be very different than simply showing it's function
/// and signature.
pub enum NGLDataKind {
    Functions,
    Type,
    Options,
    Guides,
    Packages,
    Examples,
}
