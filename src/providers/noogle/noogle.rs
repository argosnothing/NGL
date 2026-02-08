use super::schema::NoogleResponse;
use crate::{
    providers::{Provider, noogle::schema::Doc},
    schema::NGLRequest,
};

pub struct Noogle {}
static ENDPOINT_URL: &str = "https://noogle.dev/api/v1/data";

impl Provider for Noogle {
    async fn pull_data(request: NGLRequest) -> crate::schema::NGLResponse {
        let response = reqwest::get(ENDPOINT_URL)
            .await
            .expect("Failed to fetch from Noogle")
            .json::<NoogleResponse>()
            .await
            .expect("Failed to parse Noogle response");
    }

    fn get_name() -> String {
        "noogle".to_owned()
    }
}
