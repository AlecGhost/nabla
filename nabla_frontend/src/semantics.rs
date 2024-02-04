use std::collections::HashMap;
use crate::{eval::Value, GlobalIdent, ModuleAst};
use error::{Error, ErrorMessage};

mod error;
#[cfg(test)]
mod tests;
pub mod types;
pub mod values;

pub type SymbolTable = HashMap<GlobalIdent, Value>;

/// Analyze the semantics of the module.
///
/// The analysis is split into two parts:
///
/// 1. Type analysis
/// 2. Value analysis
///
/// The analyses are executed in order and their errors accumulated.
///
/// This function returns (_init values_, _symbol table_, _errors_).
pub fn analyze(module_ast: &ModuleAst) -> (Vec<Value>, SymbolTable, Vec<Error>) {
    let type_info = types::analyze(module_ast);
    let mut errors = type_info.errors;
    let (inits, symbol_table, value_errors) = values::analyze(module_ast);
    errors.extend(value_errors);
    (inits, symbol_table, errors)
}
