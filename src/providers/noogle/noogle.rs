use crate::providers::Provider;

pub struct Noogle {}
static ENDPOINT_URL: &str = "https://noogle.dev/api/v1/data";

impl Provider for Noogle {
    // TODO: We can't do this till we move the structs from noogle-search over here to map properly
    async fn pull_data() -> crate::schema::NGLResponse {
        todo!()
    }

    fn get_name() -> String {
        "noogle".to_owned()
    }
}
