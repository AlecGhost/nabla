use crate::{
    eval::Value,
    semantics::{
        error::{Error, ErrorMessage},
        namespace::Binding,
    },
    GlobalIdent, ModuleAst,
};
use std::collections::HashMap;

use self::{namespace::NamespaceResult, values::ValuesResult};

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
/// Global identifiers and their binding type
type BindingMap = HashMap<GlobalIdent, Binding>;
type Errors = Vec<Error>;

#[derive(Clone, Debug)]
pub struct SemanticsResult {
    pub inits: Vec<Value>,
    pub symbol_table: SymbolTable,
    pub errors: Errors,
}

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
pub fn analyze(module_ast: &ModuleAst) -> SemanticsResult {
    let (uses, mut errors) = uses::analyze(module_ast);
    let NamespaceResult {
        namespace,
        bindings,
        errors: namespace_errors,
    } = namespace::analyze(&uses, module_ast);
    errors.extend(namespace_errors);
    let types_result = types::analyze(module_ast, &namespace, &bindings);
    errors.extend(types_result.errors);
    let ValuesResult {
        inits,
        symbol_table,
        errors: value_errors,
    } = values::analyze(module_ast);
    errors.extend(value_errors);

    SemanticsResult {
        inits,
        symbol_table,
        errors,
    }
}
