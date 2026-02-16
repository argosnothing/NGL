#![allow(unused)]
use crate::db::{
    entities::{example, function, guide, option, package, r#type},
    services::insert,
};
use sea_orm::{DatabaseConnection, DbErr};
use tokio::sync::mpsc;

const BATCH_SIZE: usize = 150;

/// Carrier for event data.
pub enum ProviderEvent {
    Function(function::ActiveModel),
    Example(example::ActiveModel),
    Guide(guide::ActiveModel),
    Option(option::ActiveModel),
    Package(package::ActiveModel),
    Type(r#type::ActiveModel),
}

pub struct EventChannel {
    sender: mpsc::Sender<ProviderEvent>,
}

impl EventChannel {
    /// Send a bit of data. Awaits if the channel buffer is full (backpressure).
    /// # Examples:
    /// ```ignore 
    ///   channel.send(ProviderEvent::Function(function::ActiveModel {
    ///    id: NotSet,
    ///    name: Set(doc.meta.title.clone()),
    ///    provider_name: Set("noogle".to_owned()),
    ///    format: Set(DocumentationFormat::Markdown),
    ///    signature: Set(doc.meta.signature.clone()),
    ///    data: Set(content.clone()),
    /// })).await;
    /// ```
    pub async fn send(&self, event: ProviderEvent) {
        // Ignore send errors - channel closed means consumer panicked
        let _ = self.sender.send(event).await;
    }
}

/// Creates an EventChannel and spawns a background consumer task.
/// Returns the channel and a handle to await completion.
pub fn create_event_channel(
    db: DatabaseConnection,
) -> (EventChannel, tokio::task::JoinHandle<Result<(), DbErr>>) {
    let (sender, receiver) = mpsc::channel(BATCH_SIZE * 2);
    let handle = tokio::spawn(batch_consumer(receiver, db));
    (EventChannel { sender }, handle)
}

/// Background task that consumes events, batches them, and inserts to DB.
async fn batch_consumer(
    mut receiver: mpsc::Receiver<ProviderEvent>,
    db: DatabaseConnection,
) -> Result<(), DbErr> {
    let mut functions: Vec<function::ActiveModel> = Vec::new();
    let mut examples: Vec<example::ActiveModel> = Vec::new();
    let mut guides: Vec<guide::ActiveModel> = Vec::new();
    let mut options: Vec<option::ActiveModel> = Vec::new();
    let mut packages: Vec<package::ActiveModel> = Vec::new();
    let mut types: Vec<r#type::ActiveModel> = Vec::new();

    while let Some(event) = receiver.recv().await {
        match event {
            ProviderEvent::Function(model) => {
                functions.push(model);
                if functions.len() >= BATCH_SIZE {
                    insert(&db, functions.drain(..).collect()).await?;
                }
            }
            ProviderEvent::Example(model) => {
                examples.push(model);
                if examples.len() >= BATCH_SIZE {
                    insert(&db, examples.drain(..).collect()).await?;
                }
            }
            ProviderEvent::Guide(model) => {
                guides.push(model);
                if guides.len() >= BATCH_SIZE {
                    insert(&db, guides.drain(..).collect()).await?;
                }
            }
            ProviderEvent::Option(model) => {
                options.push(model);
                if options.len() >= BATCH_SIZE {
                    insert(&db, options.drain(..).collect()).await?;
                }
            }
            ProviderEvent::Package(model) => {
                packages.push(model);
                if packages.len() >= BATCH_SIZE {
                    insert(&db, packages.drain(..).collect()).await?;
                }
            }
            ProviderEvent::Type(model) => {
                types.push(model);
                if types.len() >= BATCH_SIZE {
                    insert(&db, types.drain(..).collect()).await?;
                }
            }
        }
    }

    // Flush remainders
    if !functions.is_empty() {
        insert(&db, functions).await?;
    }
    if !examples.is_empty() {
        insert(&db, examples).await?;
    }
    if !guides.is_empty() {
        insert(&db, guides).await?;
    }
    if !options.is_empty() {
        insert(&db, options).await?;
    }
    if !packages.is_empty() {
        insert(&db, packages).await?;
    }
    if !types.is_empty() {
        insert(&db, types).await?;
    }

    Ok(())
}
