use crate::NGLDataKind;

pub mod example;
pub mod function;
pub mod guide;
pub mod option;
pub mod package;
pub mod provider;
pub mod provider_kind_cache;
pub mod r#type;

pub trait NGLDataEntity: sea_orm::ActiveModelTrait {
    const KIND: NGLDataKind;
}
