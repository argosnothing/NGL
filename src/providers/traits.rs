use crate::db::entities::{example, function};

pub trait ProvidesFunctions {
    fn get_functions(&self) -> Vec<function::ActiveModel>;
}

pub trait ProvidesExamples {
    fn get_examples(&self) -> Vec<example::ActiveModel>;
}
