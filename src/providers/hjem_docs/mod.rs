use crate::providers::{Provider, EventChannel};
use crate::schema::NGLDataKind;
use async_trait::async_trait;
use sea_orm::DbErr;

#[allow(unused)]
pub struct HjemDocs;

#[async_trait]
impl Provider for HjemDocs {
    fn get_info(&self) -> super::ProviderInformation {
        todo!()
    }

    async fn sync(&mut self, _channel: &EventChannel, _kinds: &[NGLDataKind]) -> Result<(), DbErr> {
        todo!()
    }
}
