use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::collections::HashMap;
use std::io::IsTerminal;
use tokio::sync::broadcast;

use crate::providers::channel::{CountsSnapShot, StatusEvent};

#[derive(Clone)]
pub struct SyncProgress {
    mp: MultiProgress,
    is_tty: bool,
}

impl SyncProgress {
    pub fn new() -> Self {
        let is_tty = std::io::stderr().is_terminal();
        Self {
            mp: MultiProgress::new(),
            is_tty,
        }
    }

    fn add_provider(&self, provider: &str) -> ProgressBar {
        if !self.is_tty {
            return ProgressBar::hidden();
        }

        let pb = self.mp.add(ProgressBar::new_spinner());

        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {prefix}: {msg}")
                .unwrap(),
        );

        pb.set_prefix(provider.to_string());
        pb.set_message("starting...");
        pb.enable_steady_tick(std::time::Duration::from_millis(100));

        pb
    }
}

pub async fn run_progress_ui(mut rx: broadcast::Receiver<StatusEvent>) {
    let progress = SyncProgress::new();

    let mut bars: HashMap<String, ProgressBar> = HashMap::new();
    let mut latest: HashMap<String, CountsSnapShot> = HashMap::new();

    while let Ok(event) = rx.recv().await {
        match event {
            StatusEvent::ProviderStarted { provider } => {
                let pb = progress.add_provider(&provider);
                bars.insert(provider, pb);
            }
            StatusEvent::Counts { provider, counts } => {
                latest.insert(provider.clone(), counts);

                if progress.is_tty {
                    let pb = bars
                        .entry(provider.clone())
                        .or_insert_with(|| progress.add_provider(&provider));

                    pb.set_message(format_counts(counts));
                }
            }
            StatusEvent::Message { provider, msg } => {
                if progress.is_tty {
                    if let Some(pb) = bars.get(&provider) {
                        pb.set_message(msg);
                    }
                }
            }

            StatusEvent::ProviderFinished { provider, counts } => {
                let pb = bars
                    .entry(provider.clone())
                    .or_insert_with(|| progress.add_provider(&provider));

                pb.set_style(
                    ProgressStyle::with_template("\x1b[32m✔\x1b[0m {prefix}: {msg}").unwrap(),
                );
                pb.finish_with_message(format_counts(counts));
            }
        }
    }

    if !progress.is_tty {
        for (provider, counts) in latest {
            eprintln!("{}: {}", provider, format_counts(counts));
        }
    }
}

fn format_counts(c: CountsSnapShot) -> String {
    let mut parts = Vec::new();

    if c.functions > 0 {
        parts.push(format!("{} functions", c.functions));
    }
    if c.examples > 0 {
        parts.push(format!("{} examples", c.examples));
    }
    if c.guides > 0 {
        parts.push(format!("{} guides", c.guides));
    }
    if c.options > 0 {
        parts.push(format!("{} options", c.options));
    }
    if c.packages > 0 {
        parts.push(format!("{} packages", c.packages));
    }
    if c.types > 0 {
        parts.push(format!("{} types", c.types));
    }

    if parts.is_empty() {
        "syncing…".into()
    } else {
        parts.join(", ")
    }
}
