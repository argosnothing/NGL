use crate::schema::NGLResponse;

pub mod noogle;

pub trait Provider {
    fn pull_data() -> NGLResponse;
    fn get_name() -> String;
}
