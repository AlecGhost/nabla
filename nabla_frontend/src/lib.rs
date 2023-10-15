#![warn(clippy::nursery)]
pub mod ast;
pub mod lexer;
pub mod node;
pub mod parser;
pub mod semantics;
pub mod token;

#[derive(Clone, Debug)]
pub struct GlobalIdent {
    pub root: String,
    pub path: Vec<String>,
}
