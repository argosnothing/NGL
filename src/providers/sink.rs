#![allow(unused)]
use crate::db::{
    entities::{example, function, guide, option, package, r#type},
    services::insert,
};
use async_trait::async_trait;
use sea_orm::{DatabaseConnection, DbErr};
use tokio::sync::Mutex;


/// Carrier for event data.
pub enum ProviderEvent {
    Function(function::ActiveModel),
    Example(example::ActiveModel),
    Guide(guide::ActiveModel),
    Option(option::ActiveModel),
    Package(package::ActiveModel),
    Type(r#type::ActiveModel),
}

#[async_trait]
pub trait Sink: Send + Sync {
    /// Emit an event to the sink
    /// # Examples: 
    /// ```rust
    ///   sink.emit(ProviderEvent::Function(function::ActiveModel {
    ///    id: NotSet,
    ///    name: Set(doc.meta.title.clone()),
    ///    provider_name: Set("noogle".to_owned()),
    ///    format: Set(DocumentationFormat::Markdown),
    ///    signature: Set(doc.meta.signature.clone()),
    ///    data: Set(content.clone()),
    /// }))
    /// .await?;
    /// ```
    async fn emit(&self, event: ProviderEvent) -> Result<(), DbErr>;
    async fn flush(&self) -> Result<(), DbErr>;
}

/// Sink for db, this receives events from providers and has its own buffer to batch insert in db.
pub struct DbSink {
    db: DatabaseConnection,
    functions: Mutex<Vec<function::ActiveModel>>,
    examples: Mutex<Vec<example::ActiveModel>>,
    guides: Mutex<Vec<guide::ActiveModel>>,
    options: Mutex<Vec<option::ActiveModel>>,
    packages: Mutex<Vec<package::ActiveModel>>,
    types: Mutex<Vec<r#type::ActiveModel>>,
}

impl DbSink {
    pub fn new(db: DatabaseConnection) -> Self {
        Self {
            db,
            functions: Mutex::new(Vec::new()),
            examples: Mutex::new(Vec::new()),
            guides: Mutex::new(Vec::new()),
            options: Mutex::new(Vec::new()),
            packages: Mutex::new(Vec::new()),
            types: Mutex::new(Vec::new()),
        }
    }
}

/// TODO: Potentially let a concrete provider override this somehow.
const BATCH_SIZE: usize = 150;

/// Insertion bit, we don't want to spam inserts, so fill buffer, and if it's at that size shoot em in
#[async_trait]
impl Sink for DbSink {
    async fn emit(&self, event: ProviderEvent) -> Result<(), DbErr> {
        match event {
            ProviderEvent::Function(model) => {
                let mut buf = self.functions.lock().await;
                buf.push(model);
                if buf.len() >= BATCH_SIZE {
                    let batch: Vec<_> = buf.drain(..).collect();
                    drop(buf);
                    insert(&self.db, batch).await?;
                }
            }
            ProviderEvent::Example(model) => {
                let mut buf = self.examples.lock().await;
                buf.push(model);
                if buf.len() >= BATCH_SIZE {
                    let batch: Vec<_> = buf.drain(..).collect();
                    drop(buf);
                    insert(&self.db, batch).await?;
                }
            }
            ProviderEvent::Guide(model) => {
                let mut buf = self.guides.lock().await;
                buf.push(model);
                if buf.len() >= BATCH_SIZE {
                    let batch: Vec<_> = buf.drain(..).collect();
                    drop(buf);
                    insert(&self.db, batch).await?;
                }
            }
            ProviderEvent::Option(model) => {
                let mut buf = self.options.lock().await;
                buf.push(model);
                if buf.len() >= BATCH_SIZE {
                    let batch: Vec<_> = buf.drain(..).collect();
                    drop(buf);
                    insert(&self.db, batch).await?;
                }
            }
            ProviderEvent::Package(model) => {
                let mut buf = self.packages.lock().await;
                buf.push(model);
                if buf.len() >= BATCH_SIZE {
                    let batch: Vec<_> = buf.drain(..).collect();
                    drop(buf);
                    insert(&self.db, batch).await?;
                }
            }
            ProviderEvent::Type(model) => {
                let mut buf = self.types.lock().await;
                buf.push(model);
                if buf.len() >= BATCH_SIZE {
                    let batch: Vec<_> = buf.drain(..).collect();
                    drop(buf);
                    insert(&self.db, batch).await?;
                }
            }
        }
        Ok(())
    }

    /// remainer stuff after batch 
    async fn flush(&self) -> Result<(), DbErr> {
        let funcs: Vec<_> = self.functions.lock().await.drain(..).collect();
        if !funcs.is_empty() {
            insert(&self.db, funcs).await?;
        }
        let exs: Vec<_> = self.examples.lock().await.drain(..).collect();
        if !exs.is_empty() {
            insert(&self.db, exs).await?;
        }
        let gds: Vec<_> = self.guides.lock().await.drain(..).collect();
        if !gds.is_empty() {
            insert(&self.db, gds).await?;
        }
        let opts: Vec<_> = self.options.lock().await.drain(..).collect();
        if !opts.is_empty() {
            insert(&self.db, opts).await?;
        }
        let pkgs: Vec<_> = self.packages.lock().await.drain(..).collect();
        if !pkgs.is_empty() {
            insert(&self.db, pkgs).await?;
        }
        let tps: Vec<_> = self.types.lock().await.drain(..).collect();
        if !tps.is_empty() {
            insert(&self.db, tps).await?;
        }
        Ok(())
    }
}
