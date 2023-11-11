#![warn(clippy::nursery)]

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

#[derive(Clone, Debug)]
pub struct GlobalIdent {
    pub root: String,
    pub path: Vec<String>,
}
