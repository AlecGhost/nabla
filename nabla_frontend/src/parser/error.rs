use crate::token::{TokenRange, TokenStream};
use thiserror::Error;

/// Syntax error
/// Contains an error message and the token range, where the error occurred.
#[derive(Clone, Debug, Error, PartialEq, Eq)]
#[error("{message}")]
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
    ExpectedIdent,
    ExpectedUseKind,
    ExpectedEQ,
    ExpectedExpr,
    ExpectedSingle,
    MissingClosingCurly,
    MissingClosingBracket,
    TokensAfterEof,
    UnexpectedTokens,
}

impl std::fmt::Display for ErrorMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::ExpectedIdent => "expected ident",
            Self::ExpectedUseKind => "expected use expression",
            Self::ExpectedEQ => "expected `=`",
            Self::ExpectedExpr => "expected expression",
            Self::ExpectedSingle => "expected only a single expression",
            Self::MissingClosingCurly => "missing closing `}`",
            Self::MissingClosingBracket => "missing closing `]`",
            Self::TokensAfterEof => "EOF was not the last provided token",
            Self::UnexpectedTokens => "unexpected tokens",
        };
        write!(f, "{}", message)
    }
}

#[derive(Clone, Debug)]
pub struct ParserError<'a> {
    pub kind: ParserErrorKind,
    pub input: TokenStream<'a>,
}

#[derive(Clone, Debug)]
pub enum ParserErrorKind {
    Token,
    Expect,
    IgnoreUntil,
    Nom(nom::error::ErrorKind),
}

impl<'a> nom::error::ParseError<TokenStream<'a>> for ParserError<'a> {
    fn append(_: TokenStream, _: nom::error::ErrorKind, other: Self) -> Self {
        other
    }

    fn from_error_kind(input: TokenStream<'a>, kind: nom::error::ErrorKind) -> Self {
        Self {
            kind: ParserErrorKind::Nom(kind),
            input,
        }
    }
}
