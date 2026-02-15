use crate::providers::{Provider, Sink};
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

    async fn sync(&mut self, _sink: &dyn Sink, _kinds: &[NGLDataKind]) -> Result<(), DbErr> {
        todo!()
    }
}
