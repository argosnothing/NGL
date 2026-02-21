#![allow(unused)]

use crate::db::{
    entities::{example, function, guide, guide_xref, option, package, r#type},
    services::insert,
};
use sea_orm::{ActiveValue::Set, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter};
use tokio::sync::{broadcast, mpsc};

const BATCH_SIZE: usize = 300;

#[derive(Debug, Clone, Copy, Default)]
pub struct CountsSnapShot {
    pub functions: usize,
    pub examples: usize,
    pub guides: usize,
    pub options: usize,
    pub packages: usize,
    pub types: usize,
}

#[derive(Debug, Clone)]
pub enum StatusEvent {
    ProviderStarted {
        provider: String,
    },
    Counts {
        provider: String,
        counts: CountsSnapShot,
    },
    Message {
        provider: String,
        msg: String,
    },
    ProviderFinished {
        provider: String,
        counts: CountsSnapShot,
    },
}

pub enum ProviderEvent {
    Function(function::ActiveModel),
    Example(example::ActiveModel),
    Guide(guide::ActiveModel),
    GuideXref(String, String),
    Option(option::ActiveModel),
    Package(package::ActiveModel),
    Type(r#type::ActiveModel),
}

#[derive(Clone)]
pub struct EventChannel {
    sender: mpsc::Sender<ProviderEvent>,
    status: broadcast::Sender<StatusEvent>,
}

impl EventChannel {
    pub async fn send(&self, event: ProviderEvent) {
        let _ = self.sender.send(event).await;
    }

    pub fn status(&self, ev: StatusEvent) {
        let _ = self.status.send(ev);
    }

    pub fn subscribe_status(&self) -> broadcast::Receiver<StatusEvent> {
        self.status.subscribe()
    }
}

pub fn create_event_channel(
    provider_name: String,
    db: DatabaseConnection,
    status_tx: broadcast::Sender<StatusEvent>,
) -> (EventChannel, tokio::task::JoinHandle<Result<(), DbErr>>) {
    let (sender, receiver) = mpsc::channel(BATCH_SIZE * 2);

    let handle = tokio::spawn(batch_consumer(
        provider_name.clone(),
        receiver,
        db,
        status_tx.clone(),
    ));

    (
        EventChannel {
            sender,
            status: status_tx,
        },
        handle,
    )
}

fn send_counts(status: &broadcast::Sender<StatusEvent>, provider: &str, counts: CountsSnapShot) {
    let _ = status.send(StatusEvent::Counts {
        provider: provider.to_string(),
        counts,
    });
}

async fn batch_consumer(
    provider_name: String,
    mut receiver: mpsc::Receiver<ProviderEvent>,
    db: DatabaseConnection,
    status: broadcast::Sender<StatusEvent>,
) -> Result<(), DbErr> {
    let _ = status.send(StatusEvent::ProviderStarted {
        provider: provider_name.clone(),
    });

    let mut functions: Vec<function::ActiveModel> = Vec::new();
    let mut examples: Vec<example::ActiveModel> = Vec::new();
    let mut guides: Vec<guide::ActiveModel> = Vec::new();
    let mut options: Vec<option::ActiveModel> = Vec::new();
    let mut packages: Vec<package::ActiveModel> = Vec::new();
    let mut types: Vec<r#type::ActiveModel> = Vec::new();
    let mut guide_xrefs: Vec<(String, String)> = Vec::new();

    let mut counts = CountsSnapShot::default();

    while let Some(event) = receiver.recv().await {
        match event {
            ProviderEvent::Function(model) => {
                functions.push(model);
                if functions.len() >= BATCH_SIZE {
                    let n = functions.len();
                    insert(&db, functions.drain(..).collect()).await?;
                    counts.functions += n;
                    send_counts(&status, &provider_name, counts);
                }
            }

            ProviderEvent::Example(model) => {
                examples.push(model);
                if examples.len() >= BATCH_SIZE {
                    let n = examples.len();
                    insert(&db, examples.drain(..).collect()).await?;
                    counts.examples += n;
                    send_counts(&status, &provider_name, counts);
                }
            }

            ProviderEvent::Guide(model) => {
                guides.push(model);
                if guides.len() >= BATCH_SIZE {
                    let n = guides.len();
                    insert(&db, guides.drain(..).collect()).await?;
                    counts.guides += n;
                    send_counts(&status, &provider_name, counts);
                }
            }

            ProviderEvent::GuideXref(parent_link, child_link) => {
                guide_xrefs.push((parent_link, child_link));
            }

            ProviderEvent::Option(model) => {
                options.push(model);
                if options.len() >= BATCH_SIZE {
                    let n = options.len();
                    insert(&db, options.drain(..).collect()).await?;
                    counts.options += n;
                    send_counts(&status, &provider_name, counts);
                }
            }

            ProviderEvent::Package(model) => {
                packages.push(model);
                if packages.len() >= BATCH_SIZE {
                    let n = packages.len();
                    insert(&db, packages.drain(..).collect()).await?;
                    counts.packages += n;
                    send_counts(&status, &provider_name, counts);
                }
            }

            ProviderEvent::Type(model) => {
                types.push(model);
                if types.len() >= BATCH_SIZE {
                    let n = types.len();
                    insert(&db, types.drain(..).collect()).await?;
                    counts.types += n;
                    send_counts(&status, &provider_name, counts);
                }
            }
        }
    }

    if !functions.is_empty() {
        let n = functions.len();
        insert(&db, functions).await?;
        counts.functions += n;
        send_counts(&status, &provider_name, counts);
    }

    if !examples.is_empty() {
        let n = examples.len();
        insert(&db, examples).await?;
        counts.examples += n;
        send_counts(&status, &provider_name, counts);
    }

    if !guides.is_empty() {
        let n = guides.len();
        insert(&db, guides).await?;
        counts.guides += n;
        send_counts(&status, &provider_name, counts);
    }

    if !options.is_empty() {
        let n = options.len();
        insert(&db, options).await?;
        counts.options += n;
        send_counts(&status, &provider_name, counts);
    }

    if !packages.is_empty() {
        let n = packages.len();
        insert(&db, packages).await?;
        counts.packages += n;
        send_counts(&status, &provider_name, counts);
    }

    if !types.is_empty() {
        let n = types.len();
        insert(&db, types).await?;
        counts.types += n;
        send_counts(&status, &provider_name, counts);
    }

    send_counts(&status, &provider_name, counts);

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
                guide_xref::Entity::insert(guide_xref::ActiveModel {
                    guide_id: Set(p.id),
                    sub_guide_id: Set(c.id),
                })
                .exec(&db)
                .await?;
            }
        }
    }

    let _ = status.send(StatusEvent::ProviderFinished {
        provider: provider_name.clone(),
        counts,
    });
    Ok(())
}
