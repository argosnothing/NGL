use crate::providers::SyncCounts;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::io::IsTerminal;
use std::sync::Arc;

#[derive(Clone)]
pub struct SyncProgress {
    mp: MultiProgress,
    is_tty: bool,
}

impl Default for SyncProgress {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncProgress {
    pub fn new() -> Self {
        let is_tty = std::io::stderr().is_terminal();
        Self {
            mp: MultiProgress::new(),
            is_tty,
        }
    }

    pub fn add_provider(&self, provider_name: &str) -> ProviderProgress {
        let pb = if self.is_tty {
            let pb = self.mp.add(ProgressBar::new_spinner());
            pb.set_style(
                ProgressStyle::default_spinner()
                    .template("{spinner:.green} {prefix}: {msg}")
                    .unwrap(),
            );
            pb.set_prefix(provider_name.to_string());
            pb.set_message("starting...");
            pb.enable_steady_tick(std::time::Duration::from_millis(100));
            pb
        } else {
            ProgressBar::hidden()
        };

        ProviderProgress {
            counts: Arc::new(SyncCounts::default()),
            pb,
        }
    }
}

#[derive(Clone)]
pub struct ProviderProgress {
    pub counts: Arc<SyncCounts>,
    pb: ProgressBar,
}

impl ProviderProgress {
    pub fn update(&self) {
        self.pb.set_message(self.counts.format());
    }

    pub fn finish(&self) {
        self.pb
            .set_message(format!("{} (done)", self.counts.format()));
        self.pb.finish();
    }
}
