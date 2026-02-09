use crate::db::entities::{example, function};

pub trait ProvidesFunctions<T> {
    fn get_functions(data: &T) -> Vec<function::ActiveModel>;
}

pub trait ProvidesExamples<T> {
    fn get_examples(data: &T) -> Vec<example::ActiveModel>;
}
