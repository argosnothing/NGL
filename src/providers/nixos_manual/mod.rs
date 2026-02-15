use std::{i32, sync::Arc};

use async_trait::async_trait;
use scraper::{Element, ElementRef, Html, Selector};
use sea_orm::DbErr;

use crate::{
    NGLDataKind,
    providers::{Provider, ProviderInformation, Sink},
    utils::fetch_source,
};

static URL: &str = "https://nixos.org/manual/nixos/stable/";

#[derive(Default)]
pub struct NixosManual {}

#[async_trait]
impl Provider for NixosManual {
    async fn sync(&mut self, sink: Arc<dyn Sink>, kinds: &[NGLDataKind]) -> Result<(), DbErr> {
        todo!()
    }

    fn get_info(&self) -> ProviderInformation {
        ProviderInformation {
            name: "nixos_manual".to_string(),
            source: URL.to_string(),
            kinds: vec![NGLDataKind::Guide],
            sync_interval_hours: Some(u32::MAX),
        }
    }
}
