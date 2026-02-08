use crate::schema::{NGLRequest, NGLResponse};

pub mod noogle;
pub mod traits;

pub trait Provider {
    async fn pull_data(request: NGLRequest) -> NGLResponse;
    fn get_name() -> String;
}
