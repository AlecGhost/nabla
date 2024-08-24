use crate::{
    eval::Value,
    semantics::error::{Error, ErrorMessage},
    GlobalIdent, ModuleAst,
};
use std::collections::HashMap;

mod error;
pub mod namespace;
#[cfg(test)]
mod tests;
pub mod types;
pub mod uses;
pub mod values;

pub type SymbolTable = HashMap<GlobalIdent, Value>;
/// Valid identifiers and their global names
type Namespace = HashMap<String, GlobalIdent>;
type Errors = Vec<Error>;

/// Analyze the semantics of the module.
///
/// The analysis is split into four parts:
///
/// 1. Use analysis
/// 2. Namespace analysis
/// 3. Type analysis
/// 4. Value analysis
///
/// The analyses are executed in order and their errors accumulated.
///
/// This function returns (_init values_, _symbol table_, _errors_).
pub fn analyze(module_ast: &ModuleAst) -> (Vec<Value>, SymbolTable, Errors) {
    let (uses, mut errors) = uses::analyze(module_ast);
    let (namespace, namespace_errors) = namespace::analyze(&uses, module_ast);
    errors.extend(namespace_errors);
    let type_info = types::analyze(module_ast, &namespace);
    errors.extend(type_info.errors);
    let (inits, symbol_table, value_errors) = values::analyze(module_ast);
    errors.extend(value_errors);
    (inits, symbol_table, errors)
}
