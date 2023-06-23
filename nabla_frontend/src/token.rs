use std::fmt::Display;

pub const LBRACKET: &str = "[";
pub const RBRACKET: &str = "]";
pub const LCURLY: &str = "{";
pub const RCURLY: &str = "}";
pub const DOUBLE_COLON: &str = "::";
pub const STAR: &str = "*";
pub const PIPE: &str = "|";
pub const EQ: &str = "=";
pub const COLON: &str = ":";
pub const USE: &str = "use";
pub const DEF: &str = "def";
pub const LET: &str = "let";
pub const TRUE: &str = "true";
pub const FALSE: &str = "false";

type TextRange = std::ops::Range<usize>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TokenType {
    LBracket,
    RBracket,
    LCurly,
    RCurly,
    DoubleColon,
    Star,
    Pipe,
    Eq,
    Colon,
    Use,
    Def,
    Let,
    True,
    False,
    String(String),
    Char(String),
    Number(String),
    Ident(String),
    Whitespace(String),
    Comment(String),
    Unknown(String),
    Eof,
}

/// Lexical error message
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ErrorMessage {
    MissingClosingSingleQuote,
    Unknown,
}

/// Lexical error
/// Contains an error message and the text range, where the error occurred.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    pub message: ErrorMessage,
    pub range: TextRange,
}

/// A token, defined by its token type and text range.
/// Also contains errors that occurred during lexical analysis of the token.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Token {
    pub token_type: TokenType,
    pub range: TextRange,
    pub errors: Vec<Error>,
}

impl Token {
    pub const fn new(token_type: TokenType, range: TextRange) -> Self {
        Self {
            token_type,
            range,
            errors: Vec::new(),
        }
    }

    pub fn append_error(self, error: Error) -> Self {
        let mut errors = self.errors;
        errors.push(error);
        Self { errors, ..self }
    }
}

impl Error {
    pub const fn new(message: ErrorMessage, range: TextRange) -> Self {
        Self { message, range }
    }
}

impl TokenType {
    pub const fn as_static_str(&self) -> Option<&'static str> {
        use TokenType::*;
        match self {
            LBracket => Some(LBRACKET),
            RBracket => Some(RBRACKET),
            LCurly => Some(LCURLY),
            RCurly => Some(RCURLY),
            DoubleColon => Some(DOUBLE_COLON),
            Star => Some(STAR),
            Pipe => Some(PIPE),
            Eq => Some(EQ),
            Colon => Some(COLON),
            Use => Some(USE),
            Def => Some(DEF),
            Let => Some(LET),
            True => Some(TRUE),
            False => Some(FALSE),
            Eof => Some(""),
            _ => None,
        }
    }
}

impl Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use TokenType::*;
        let token_string = match self {
            Ident(s) | String(s) | Char(s) | Number(s) | Whitespace(s) | Comment(s)
            | Unknown(s) => s,
            static_token => static_token
                .as_static_str()
                .expect("Static representation must be available"),
        };
        write!(f, "{}", token_string)
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.token_type)
    }
}
