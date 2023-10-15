use crate::ast::{Global, Program};
use error::{Error, ErrorMessage};

mod error;
#[cfg(test)]
mod tests;
mod type_analysis;

pub fn analyze(program: &Program) -> Vec<Error> {
    vec![
        type_analysis::analyze(program),
        check_multiple_inits(program),
    ]
    .concat()
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
        .map(|init| Error::new(ErrorMessage::MultipleInits, init.info().range.clone()))
        .collect()
}
