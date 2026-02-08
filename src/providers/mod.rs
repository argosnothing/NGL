use crate::schema::NGLResponse;

pub mod noogle;
pub mod traits;

pub trait Provider {
    async fn pull_data() -> NGLResponse;
    fn get_name() -> String;
}
