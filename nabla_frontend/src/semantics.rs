use crate::{
    ast::{Global, Program},
    token::ToTokenRange,
};
use error::{Error, ErrorMessage};
pub use types::{BuiltInType, Rule, TypeDescription, TypeInfo};

mod error;
#[cfg(test)]
mod tests;
mod types;
pub mod values;

pub fn analyze(program: &Program) -> TypeInfo {
    let mut type_info = types::analyze(program);
    type_info.errors.extend(check_multiple_inits(program));
    type_info
}

fn check_multiple_inits(program: &Program) -> Vec<Error> {
    program
        .globals
        .iter()
        .filter_map(|global| match global {
            Global::Init(init) => Some(init),
            _ => None,
        })
        .skip(1)
        .map(|init| Error::new(ErrorMessage::MultipleInits, init.info().to_token_range()))
        .collect()
}
