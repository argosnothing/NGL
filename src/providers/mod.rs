use crate::schema::NGLDataKind;

pub mod sink;

pub mod provider;
pub mod meta;
pub mod nixos_manual;

pub use provider::Provider;
pub use sink::{DbSink, ProviderEvent, Sink};

#[cfg(feature = "nixpkgs")]
pub mod nixpkgs;

#[cfg(feature = "noogle")]
pub mod noogle;

#[derive(Clone)]
pub struct ProviderInformation {
    /// This is what determines if your provider gets synced
    pub kinds: Vec<NGLDataKind>,
    /// Must be a unique name identifying the provider.
    pub name: String,
    /// Hours to sync this providers, defaults to 24
    pub sync_interval_hours: Option<u32>,
    /// The domain this data came from. 
    #[allow(unused)]
    pub source: String,
}
