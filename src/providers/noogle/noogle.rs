use crate::providers::Provider;

pub struct Noogle {}

impl Provider for Noogle {
    fn pull_data() -> crate::schema::NGLResponse {
        todo!()
    }

    fn get_name() -> String {
        "noogle".to_owned()
    }
}
