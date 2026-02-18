#![allow(unused)]
use crate::db::{
    entities::{
        example, function, function_example, guide, guide_example, guide_xref, option,
        option_example, package, package_example, r#type, type_example,
    },
    services::insert,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, DbErr, EntityTrait,
    QueryFilter,
};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::mpsc;

const BATCH_SIZE: usize = 300;

#[derive(Default)]
pub struct SyncCounts {
    pub functions: AtomicUsize,
    pub examples: AtomicUsize,
    pub guides: AtomicUsize,
    pub options: AtomicUsize,
    pub packages: AtomicUsize,
    pub types: AtomicUsize,
}

impl SyncCounts {
    pub fn format(&self) -> String {
        let mut parts = Vec::new();
        let functions = self.functions.load(Ordering::Relaxed);
        let examples = self.examples.load(Ordering::Relaxed);
        let guides = self.guides.load(Ordering::Relaxed);
        let options = self.options.load(Ordering::Relaxed);
        let packages = self.packages.load(Ordering::Relaxed);
        let types = self.types.load(Ordering::Relaxed);

        if functions > 0 {
            parts.push(format!("{} functions", functions));
        }
        if examples > 0 {
            parts.push(format!("{} examples", examples));
        }
        if guides > 0 {
            parts.push(format!("{} guides", guides));
        }
        if options > 0 {
            parts.push(format!("{} options", options));
        }
        if packages > 0 {
            parts.push(format!("{} packages", packages));
        }
        if types > 0 {
            parts.push(format!("{} types", types));
        }

        if parts.is_empty() {
            "syncing...".to_string()
        } else {
            parts.join(", ")
        }
    }
}

/// Carrier for event data.
pub enum ProviderEvent {
    Function(function::ActiveModel),
    FunctionWithExamples(function::ActiveModel, Vec<LinkedExample>),
    Example(example::ActiveModel),
    Guide(guide::ActiveModel),
    GuideWithExamples(guide::ActiveModel, Vec<LinkedExample>),
    /// Link a child guide to its parent
    GuideXref(String, String),
    Option(option::ActiveModel),
    OptionWithExamples(option::ActiveModel, Vec<LinkedExample>),
    Package(package::ActiveModel),
    PackageWithExamples(package::ActiveModel, Vec<LinkedExample>),
    Type(r#type::ActiveModel),
    TypeWithExamples(r#type::ActiveModel, Vec<LinkedExample>),
}

pub struct LinkedExample {
    pub placeholder_key: String,
    pub model: example::ActiveModel,
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
        let _ = self.sender.send(event).await;
    }
}

/// Creates an EventChannel and spawns a background consumer task.
/// Returns the channel and a handle to await completion.
/// If `counts` is provided, it will be updated with progress.
pub fn create_event_channel(
    db: DatabaseConnection,
    counts: Option<Arc<SyncCounts>>,
) -> (EventChannel, tokio::task::JoinHandle<Result<(), DbErr>>) {
    let (sender, receiver) = mpsc::channel(BATCH_SIZE * 2);
    let handle = tokio::spawn(batch_consumer(receiver, db, counts));
    (EventChannel { sender }, handle)
}

/// Background task that consumes events, batches them, and inserts to DB.
async fn batch_consumer(
    mut receiver: mpsc::Receiver<ProviderEvent>,
    db: DatabaseConnection,
    counts: Option<Arc<SyncCounts>>,
) -> Result<(), DbErr> {
    let mut functions: Vec<function::ActiveModel> = Vec::new();
    let mut examples: Vec<example::ActiveModel> = Vec::new();
    let mut guides: Vec<guide::ActiveModel> = Vec::new();
    let mut options: Vec<option::ActiveModel> = Vec::new();
    let mut packages: Vec<package::ActiveModel> = Vec::new();
    let mut types: Vec<r#type::ActiveModel> = Vec::new();
    let mut guide_xrefs: Vec<(String, String)> = Vec::new();

    while let Some(event) = receiver.recv().await {
        match event {
            ProviderEvent::Function(model) => {
                functions.push(model);
                if functions.len() >= BATCH_SIZE {
                    let count = functions.len();
                    insert(&db, functions.drain(..).collect()).await?;
                    if let Some(ref c) = counts {
                        c.functions.fetch_add(count, Ordering::Relaxed);
                    }
                }
            }
            ProviderEvent::Example(model) => {
                examples.push(model);
                if examples.len() >= BATCH_SIZE {
                    let count = examples.len();
                    insert(&db, examples.drain(..).collect()).await?;
                    if let Some(ref c) = counts {
                        c.examples.fetch_add(count, Ordering::Relaxed);
                    }
                }
            }
            ProviderEvent::Guide(model) => {
                guides.push(model);
                if guides.len() >= BATCH_SIZE {
                    let count = guides.len();
                    insert(&db, guides.drain(..).collect()).await?;
                    if let Some(ref c) = counts {
                        c.guides.fetch_add(count, Ordering::Relaxed);
                    }
                }
            }
            ProviderEvent::GuideXref(parent_link, child_link) => {
                // Defer processing until all guides are inserted
                guide_xrefs.push((parent_link, child_link));
            }
            ProviderEvent::Option(model) => {
                options.push(model);
                if options.len() >= BATCH_SIZE {
                    let count = options.len();
                    insert(&db, options.drain(..).collect()).await?;
                    if let Some(ref c) = counts {
                        c.options.fetch_add(count, Ordering::Relaxed);
                    }
                }
            }
            ProviderEvent::Package(model) => {
                packages.push(model);
                if packages.len() >= BATCH_SIZE {
                    let count = packages.len();
                    insert(&db, packages.drain(..).collect()).await?;
                    if let Some(ref c) = counts {
                        c.packages.fetch_add(count, Ordering::Relaxed);
                    }
                }
            }
            ProviderEvent::Type(model) => {
                types.push(model);
                if types.len() >= BATCH_SIZE {
                    let count = types.len();
                    insert(&db, types.drain(..).collect()).await?;
                    if let Some(ref c) = counts {
                        c.types.fetch_add(count, Ordering::Relaxed);
                    }
                }
            }
            ProviderEvent::FunctionWithExamples(func_model, linked_examples) => {
                let func = func_model.insert(&db).await?;
                if let Some(ref c) = counts {
                    c.functions.fetch_add(1, Ordering::Relaxed);
                }
                for linked in linked_examples {
                    let ex = linked.model.insert(&db).await?;
                    function_example::ActiveModel {
                        function_id: Set(func.id),
                        example_id: Set(ex.id),
                        placeholder_key: Set(linked.placeholder_key),
                    }
                    .insert(&db)
                    .await?;
                    if let Some(ref c) = counts {
                        c.examples.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }
            ProviderEvent::GuideWithExamples(guide_model, linked_examples) => {
                let g = guide_model.insert(&db).await?;
                if let Some(ref c) = counts {
                    c.guides.fetch_add(1, Ordering::Relaxed);
                }
                for linked in linked_examples {
                    let ex = linked.model.insert(&db).await?;
                    guide_example::ActiveModel {
                        guide_id: Set(g.id),
                        example_id: Set(ex.id),
                        placeholder_key: Set(linked.placeholder_key),
                    }
                    .insert(&db)
                    .await?;
                    if let Some(ref c) = counts {
                        c.examples.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }
            ProviderEvent::OptionWithExamples(opt_model, linked_examples) => {
                let o = opt_model.insert(&db).await?;
                if let Some(ref c) = counts {
                    c.options.fetch_add(1, Ordering::Relaxed);
                }
                for linked in linked_examples {
                    let ex = linked.model.insert(&db).await?;
                    option_example::ActiveModel {
                        option_id: Set(o.id),
                        example_id: Set(ex.id),
                        placeholder_key: Set(linked.placeholder_key),
                    }
                    .insert(&db)
                    .await?;
                    if let Some(ref c) = counts {
                        c.examples.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }
            ProviderEvent::PackageWithExamples(pkg_model, linked_examples) => {
                let p = pkg_model.insert(&db).await?;
                if let Some(ref c) = counts {
                    c.packages.fetch_add(1, Ordering::Relaxed);
                }
                for linked in linked_examples {
                    let ex = linked.model.insert(&db).await?;
                    package_example::ActiveModel {
                        package_id: Set(p.id),
                        example_id: Set(ex.id),
                        placeholder_key: Set(linked.placeholder_key),
                    }
                    .insert(&db)
                    .await?;
                    if let Some(ref c) = counts {
                        c.examples.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }
            ProviderEvent::TypeWithExamples(type_model, linked_examples) => {
                let t = type_model.insert(&db).await?;
                if let Some(ref c) = counts {
                    c.types.fetch_add(1, Ordering::Relaxed);
                }
                for linked in linked_examples {
                    let ex = linked.model.insert(&db).await?;
                    type_example::ActiveModel {
                        type_id: Set(t.id),
                        example_id: Set(ex.id),
                        placeholder_key: Set(linked.placeholder_key),
                    }
                    .insert(&db)
                    .await?;
                    if let Some(ref c) = counts {
                        c.examples.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }
        }
    }

    // Flush remainders
    if !functions.is_empty() {
        let count = functions.len();
        insert(&db, functions).await?;
        if let Some(ref c) = counts {
            c.functions.fetch_add(count, Ordering::Relaxed);
        }
    }
    if !examples.is_empty() {
        let count = examples.len();
        insert(&db, examples).await?;
        if let Some(ref c) = counts {
            c.examples.fetch_add(count, Ordering::Relaxed);
        }
    }
    if !guides.is_empty() {
        let count = guides.len();
        insert(&db, guides).await?;
        if let Some(ref c) = counts {
            c.guides.fetch_add(count, Ordering::Relaxed);
        }
    }
    if !options.is_empty() {
        let count = options.len();
        insert(&db, options).await?;
        if let Some(ref c) = counts {
            c.options.fetch_add(count, Ordering::Relaxed);
        }
    }
    if !packages.is_empty() {
        let count = packages.len();
        insert(&db, packages).await?;
        if let Some(ref c) = counts {
            c.packages.fetch_add(count, Ordering::Relaxed);
        }
    }
    if !types.is_empty() {
        let count = types.len();
        insert(&db, types).await?;
        if let Some(ref c) = counts {
            c.types.fetch_add(count, Ordering::Relaxed);
        }
    }

    if !guide_xrefs.is_empty() {
        for (parent_link, child_link) in guide_xrefs {
            let parent = guide::Entity::find()
                .filter(guide::Column::Link.eq(&parent_link))
                .one(&db)
                .await?;
            let child = guide::Entity::find()
                .filter(guide::Column::Link.eq(&child_link))
                .one(&db)
                .await?;

            if let (Some(p), Some(c)) = (parent, child) {
                let _ = guide_xref::ActiveModel {
                    guide_id: Set(p.id),
                    sub_guide_id: Set(c.id),
                }
                .insert(&db)
                .await;
            }
        }
    }

    Ok(())
}
