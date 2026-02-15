use std::sync::Arc;

use async_trait::async_trait;
use sea_orm::DbErr;

use crate::{
    NGLDataKind,
    providers::{Provider, ProviderInformation, Sink},
};

#[derive(Default)]
pub struct NixosManual {}

#[async_trait]
impl Provider for NixosManual {
    async fn sync(&mut self, sink: Arc<dyn Sink>, kinds: &[NGLDataKind]) -> Result<(), DbErr> {
        todo!()
    }

    fn get_info(&self) -> ProviderInformation {
        todo!()
    }
}
