use crate::NGLDataKind;

pub mod example;
pub mod function;
pub mod function_example;
pub mod guide;
pub mod guide_example;
pub mod guide_xref;
pub mod option;
pub mod option_example;
pub mod package;
pub mod package_example;
pub mod provider;
pub mod provider_kind_cache;
pub mod r#type;
pub mod type_example;

pub trait NGLDataEntity: sea_orm::ActiveModelTrait {
    #[allow(unused)]
    const KIND: NGLDataKind;
}
