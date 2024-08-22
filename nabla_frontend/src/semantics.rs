use crate::{
    eval::Value,
    semantics::error::{Error, ErrorMessage},
    GlobalIdent, ModuleAst,
};
use std::collections::HashMap;

mod error;
#[cfg(test)]
mod tests;
pub mod types;
pub mod uses;
pub mod values;

pub type SymbolTable = HashMap<GlobalIdent, Value>;

/// Analyze the semantics of the module.
///
/// The analysis is split into three parts:
///
/// 1. Use analysis
/// 2. Type analysis
/// 3. Value analysis
///
/// The analyses are executed in order and their errors accumulated.
///
/// This function returns (_init values_, _symbol table_, _errors_).
pub fn analyze(
    module_ast: &ModuleAst,
    extern_table: &SymbolTable,
) -> (Vec<Value>, SymbolTable, Vec<Error>) {
    let (uses, mut errors) = uses::analyze(module_ast);
    let type_info = types::analyze(module_ast);
    errors.extend(type_info.errors);
    let (inits, symbol_table, value_errors) = values::analyze(module_ast, extern_table, &uses);
    errors.extend(value_errors);
    (inits, symbol_table, errors)
}
