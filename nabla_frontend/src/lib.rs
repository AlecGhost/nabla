#![warn(clippy::nursery)]

use ast::Ast;

#[cfg(test)]
const fn new_vec<T>(_: &Vec<T>) -> Vec<T> {
    Vec::new()
}

#[cfg(test)]
macro_rules! assert_empty {
    ($vec:expr) => {
        assert_eq!(crate::new_vec(&$vec), $vec);
    };
}

pub mod ast;
pub mod eval;
pub mod lexer;
pub mod parser;
pub mod semantics;
pub mod token;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GlobalIdent {
    pub root: String,
    pub path: Vec<String>,
}

impl GlobalIdent {
    pub fn extend(mut self, ident: String) -> Self {
        self.path.push(ident);
        self
    }
}

impl Default for GlobalIdent {
    fn default() -> Self {
        Self {
            root: "root".to_string(),
            path: Vec::new(),
        }
    }
}

impl std::fmt::Display for GlobalIdent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.path.is_empty() {
            write!(f, "{}", self.root)
        } else {
            write!(f, "{}::{}", self.root, self.path.join("::"))
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModuleAst {
    pub name: GlobalIdent,
    pub ast: Ast,
}

impl ModuleAst {
    pub const fn new(name: GlobalIdent, ast: Ast) -> Self {
        Self { name, ast }
    }
}
