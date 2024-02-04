use crate::{ast::Program, eval::Value};
use error::{Error, ErrorMessage};

mod error;
#[cfg(test)]
mod tests;
pub mod types;
pub mod values;

/// Analyze the semantics of the program.
///
/// The analysis is split into two parts:
///
/// 1. Type analysis
/// 2. Value analysis
///
/// The analyses are executed in order and their errors accumulated.
///
/// This function returns (_init values_, _symbol table_, _errors_).
pub fn analyze(program: &Program) -> (Vec<Value>, values::SymbolTable, Vec<Error>) {
    let type_info = types::analyze(program);
    let mut errors = type_info.errors;
    let (inits, symbol_table, value_errors) = values::analyze(program);
    errors.extend(value_errors);
    (inits, symbol_table, errors)
}
