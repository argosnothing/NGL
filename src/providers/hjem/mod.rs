use crate::providers::{EventChannel, Provider};
use crate::schema::NGLDataKind;
use async_trait::async_trait;
use sea_orm::DbErr;

#[allow(unused)]
pub struct HjemDocs;

static ENDPOINT_URL: &str = "https://hjem.feel-co.org/assets/search-data.json";

#[async_trait]
impl Provider for HjemDocs {
    fn get_info(&self) -> super::ProviderInformation {
        todo!()
    }

    async fn sync(&mut self, _channel: &EventChannel, _kinds: &[NGLDataKind]) -> Result<(), DbErr> {
        todo!()
    }
}
