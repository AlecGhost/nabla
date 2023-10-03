use crate::token::TokenRange;
use thiserror::Error;

/// Semantic error
/// Contains an error message and the token range, where the error occurred.
#[derive(Clone, Debug, Error, PartialEq, Eq)]
#[error("Token {}-{}: {message}", .range.start, .range.end)]
pub struct Error {
    pub message: ErrorMessage,
    pub range: TokenRange,
}

impl Error {
    pub const fn new(message: ErrorMessage, range: TokenRange) -> Self {
        Self { message, range }
    }
}

/// Syntax error message
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ErrorMessage {
    AliasMustBeString,
    AliasMustBeIdent,
    UnionInInit,
}

impl std::fmt::Display for ErrorMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::AliasMustBeString => "alias must be a string",
            Self::AliasMustBeIdent => "alias must be an identifier",
            Self::UnionInInit => "unions cannot be used in initializations",
        };
        write!(f, "{}", message)
    }
}
