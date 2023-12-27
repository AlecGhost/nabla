use crate::{
    ast::{Global, Program},
    token::ToTokenRange,
};
use error::{Error, ErrorMessage};

mod error;
#[cfg(test)]
mod tests;
pub mod types;
pub mod values;

pub fn analyze(program: &Program) -> Vec<Error> {
    let type_info = types::analyze(program);
    let mut errors = type_info.errors;
    errors.extend(check_multiple_inits(program));
    let (_, _, value_errors) = values::analyze(program);
    errors.extend(value_errors);
    errors
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
