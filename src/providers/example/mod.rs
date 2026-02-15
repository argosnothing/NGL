// Copy paste this file when starting your own provider to make
// your life a little bit easier.
#![allow(unused)]
use crate::{
    db::{
        entities::{example, function, guide, option, package, r#type},
        enums::{documentation_format::DocumentationFormat, language::Language},
    },
    providers::{Provider, ProviderEvent, ProviderInformation, Sink},
    schema::NGLDataKind,
};
use async_trait::async_trait;
use pulldown_cmark::{CodeBlockKind, Event, Parser, Tag, TagEnd};
use sea_orm::{ActiveValue::*, DbErr};
use std::sync::Arc;

// structs implementing providers are kept instantiatable as more advanced providers
// leverage their own state in certain instances. You likely won't need to use self when
// writing your provider.
#[derive(Default)]
pub struct Example;

// Unfortunately, trait expansion makes the provider trait unwieldly to auto build impl stubs for it,
// Example is to make it easier to get setup
#[async_trait]
impl Provider for Example {
    fn get_info(&self) -> ProviderInformation {
        ProviderInformation {
            name: "example".to_string(),
            source: "example.com".to_string(),
            kinds: vec![NGLDataKind::Function, NGLDataKind::Example],
            sync_interval_hours: Some(u32::MAX),
        }
    }

    /// To make your life a little easier i'll explain this bit here.
    /// NGL does a lot behind the scene to make writing code to fetch from remote sources
    /// and give it to the database easier, but if you don't know what it's doing this can be overwhelming
    ///
    /// Breath... It'll be ok.
    ///
    /// Application Flow:  
    /// 1. The registry runs `refresh` on every provider that has relevant kinds of documentation it has in its providers vec.
    /// 2. that `refresh` does a few things,
    ///    1. manages when your sync will be ran ( using a configurable sync_interval_hours )
    ///    2. adds your provider to the providers list, and providers_cache list ( one for each kind of data )
    ///    3. creates a new `sink` with its own database connection, what is this sink?
    ///       - NGL Providers use events to interact with the database, it does this through sinks, which act as a target
    ///         to emit those events to. I'll provide an example in the sync method body.
    ///       - NGL is fully async, so
    async fn sync(&mut self, sink: Arc<dyn Sink>, kinds: &[NGLDataKind]) -> Result<(), DbErr> {
        todo!();
    }
}
