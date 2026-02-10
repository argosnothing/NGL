use async_trait::async_trait;
use crate::providers::Provider;
pub mod schema;

pub struct NixPkgs {}

#[async_trait]
impl Provider for NixPkgs {
    fn get_info(&self) -> super::ProviderInformation {
        todo!()
    }

    async fn fetch_functions(&mut self) -> Vec<crate::db::entities::function::ActiveModel> {
        todo!()
    }

    async fn fetch_examples(&mut self) -> Vec<crate::db::example::ActiveModel> {
        todo!()
    }

    async fn fetch_guides(&mut self) -> Vec<crate::db::entities::guide::ActiveModel> {
        todo!()
    }

    async fn fetch_options(&mut self) -> Vec<crate::db::entities::option::ActiveModel> {
        todo!()
    }

    async fn fetch_packages(&mut self) -> Vec<crate::db::entities::package::ActiveModel> {
        todo!()
    }

    async fn fetch_types(&mut self) -> Vec<crate::db::entities::r#type::ActiveModel> {
        todo!()
    }
}
