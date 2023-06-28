use crate::token::{TokenRange, TokenStream};

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
}

/// Syntax error
/// Contains an error message and the text range, where the error occurred.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    pub message: ErrorMessage,
    pub range: TokenRange,
}

impl Error {
    pub const fn new(message: ErrorMessage, range: TokenRange) -> Self {
        Self { message, range }
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
