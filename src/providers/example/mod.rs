// not to be confused with the example `kind` :)
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

static PROVIDER_NAME: &str = "example";
// structs implementing providers are kept instantiatable as more advanced providers
// leverage their own state in certain instances. You likely won't need to use self when
// writing your provider.
#[derive(Default)]
pub struct Example;

// Unfortunately, trait expansion makes the provider trait unwieldly to auto build impl stubs for it,
// Example is to make it easier to get setup
#[async_trait]
impl Provider for Example {
    /// Lets create a provider that creates a single function in the db called banana
    fn get_info(&self) -> ProviderInformation {
        ProviderInformation {
            /// Name of the provider, needs to be unique provider name.
            name: PROVIDER_NAME.to_string(),
            /// Provided by example.com :)
            source: "example.com".to_string(),
            /// Supports providing function documentation only
            kinds: vec![NGLDataKind::Function],
            /// Syncs one time, and then virtually never syncs again :p
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
    ///    2. adds your provider to the providers list
    ///    3. builds a `kinds` vec based on the `kinds` from the `request` that need the provider to cache those kinds.
    ///       any kinds that are coming into the sync method are expected to be `emitted` at some point during the `sync`.
    ///    4. uses that `kinds` vec to *DELETE* existing kinds of data matching what will need to be cached
    ///    5. creates a new `sink` with its own database connection, what is this sink?
    ///       - NGL Providers use events to interact with the database, it does this through sinks, which act as a target
    ///         to emit those events to. I'll provide an example in the sync method body.
    ///       - One nifty feature of this Sink impl is that each providers sink has their own internal buffer,
    ///         so when you start emitting events to the sink you do not need to worry about batching your own data for the
    ///         inserts. Just emit it as soon as its ready and the `sink` will take care of the rest.
    ///    6. Finally, flushes any remaining sink buffers ( by inserting those into db ) and then adds or updates an entry
    ///       to the `provider_kinds_cache` table to track when this synced happened for the next refresh :)
    /// 3. The provider implements sync, which should
    ///    1. get data from whatever source (api endpoint, html, even a local json file)
    ///    2. based on the `kinds` of data coming in, shapes whatever it gets from its own source
    ///       to one that satisfies the `kinds` of data coming in.
    ///       If Example, Options, Types come in, you need to emit those different kinds of data to the sink,
    ///       as at this point any previously cached kinds of data will be deleted already.
    async fn sync(&mut self, sink: &dyn Sink, kinds: &[NGLDataKind]) -> Result<(), DbErr> {
        // We just emitted a database-mapped entity model to the sink, now the sink's job is to buffer that
        // and when the buffer gets full, insert the batch of models into the db!
        // Your whole goal is to shape whatever data the `kinds` param is requesting into the correlating database
        // rows. Have fun!
        sink.emit(ProviderEvent::Function(function::ActiveModel {
            id: NotSet,
            name: Set("banana".to_string()),
            provider_name: Set(PROVIDER_NAME.to_string()),
            format: Set(DocumentationFormat::Markdown),
            signature: Set(Some("x, y -> v".to_string())),
            data: Set("so much data!".to_string()),
            source_url: Set(Some("example.com".to_string())),
            source_code_url: Set(Some("some other thing preferrably with built in marker for where in the sourcecode this is :)".to_string())),
            aliases: Set(Some("what other funcs are there".to_string())),
        }));
        Ok(())
    }
}
