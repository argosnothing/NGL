use async_trait::async_trait;
use scraper::{Element, ElementRef, Html, Selector};
use sea_orm::DbErr;

use crate::{
    NGLDataKind,
    providers::{Provider, ProviderInformation, EventChannel},
    utils::fetch_source,
};

static URL: &str = "https://nixos.org/manual/nixos/stable/";

#[derive(Default)]
pub struct NixosManual {}

#[async_trait]
impl Provider for NixosManual {
    async fn sync(&mut self, _channel: &EventChannel, kinds: &[NGLDataKind]) -> Result<(), DbErr> {
        let Ok(result) = fetch_source(URL).await else {
            return Ok(());
        };

        println!("{}", result);
        Ok(())
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

// impl NixosManual {
//     fn parse_html(html: &str) ->
// }
//

// TODO: Guides can have subguides, we will need to change how a guide is structured in the db.
// a self-referential many-to-many table on guides is probably the play here
/// Oh boy... This will be fun. We
struct NixosManualGuide {}
struct NixosManualExample {
    pub source: String,
    pub data: String,
}
