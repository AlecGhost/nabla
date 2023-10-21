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
pub const AS: &str = "as";
pub const TRUE: &str = "true";
pub const FALSE: &str = "false";
pub const NULL: &str = "null";
pub const EOF: &str = "";

pub type TextRange = std::ops::Range<usize>;
pub type TokenRange = std::ops::Range<usize>;
type ParserError = crate::parser::Error;

pub trait ToTextRange {
    fn to_text_range(&self) -> TextRange;
}

pub trait ToTokenRange {
    fn to_token_range(&self) -> TokenRange;
}

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
    As,
    True,
    False,
    Null,
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
    MissingDecimals,
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
            As => Some(AS),
            True => Some(TRUE),
            False => Some(FALSE),
            Null => Some(NULL),
            Eof => Some(EOF),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TokenStream<'a> {
    tokens: &'a [Token],
    first_ptr: *const Token,
    pub error_buffer: Vec<ParserError>,
}

impl<'a> TokenStream<'a> {
    pub fn first_token(&self) -> Option<&Token> {
        self.tokens.get(0)
    }
}

impl TokenStream<'_> {
    fn distance(first: *const Token, second: *const Token) -> usize {
        // because we do pointer arithmetic, the size of `Token` in memory is needed,
        // to calculate the offset.
        let size = std::mem::size_of::<Token>();

        (second as usize - first as usize) / size
    }

    pub fn location_offset(&self) -> usize {
        TokenStream::distance(self.first_ptr, self.tokens.as_ptr())
    }

    pub const fn tokens(&self) -> &[Token] {
        self.tokens
    }

    pub fn append_error(&mut self, error: ParserError) {
        self.error_buffer.push(error);
    }
}

impl<'a> ToTokenRange for TokenStream<'a> {
    fn to_token_range(&self) -> TokenRange {
        self.first_token().map_or(0..0, |token| token.range.clone())
    }
}

impl<'a> From<&'a [Token]> for TokenStream<'a> {
    fn from(tokens: &'a [Token]) -> Self {
        Self {
            tokens,
            first_ptr: tokens.as_ptr(),
            error_buffer: Vec::new(),
        }
    }
}

/// source: [Stackoverflow](https://stackoverflow.com/a/57203324)
/// enables indexing and slicing
impl<'a, Idx> std::ops::Index<Idx> for TokenStream<'a>
where
    Idx: std::slice::SliceIndex<[Token]>,
{
    type Output = Idx::Output;

    fn index(&self, index: Idx) -> &Self::Output {
        &self.tokens[index]
    }
}

impl<'a> nom::InputLength for TokenStream<'a> {
    fn input_len(&self) -> usize {
        self.tokens.len()
    }
}

impl<'a> nom::InputTake for TokenStream<'a> {
    fn take(&self, count: usize) -> Self {
        Self {
            tokens: &self.tokens[0..count],
            ..self.clone()
        }
    }

    fn take_split(&self, count: usize) -> (Self, Self) {
        (
            Self {
                tokens: &self.tokens[count..],
                ..self.clone()
            },
            Self {
                tokens: &self.tokens[0..count],
                ..self.clone()
            },
        )
    }
}

/// source: [nom traits](https://docs.rs/nom/latest/src/nom/traits.rs.html#62-69)
impl<'a> nom::Offset for TokenStream<'a> {
    fn offset(&self, second: &Self) -> usize {
        let fst = self.tokens.as_ptr();
        let snd = second.tokens.as_ptr();
        TokenStream::distance(fst, snd)
    }
}

impl<'a> nom::Slice<std::ops::RangeTo<usize>> for TokenStream<'a> {
    fn slice(&self, range: std::ops::RangeTo<usize>) -> Self {
        Self {
            tokens: &self.tokens[range],
            ..self.clone()
        }
    }
}

/// source: [Monkey Rust lexer](https://github.com/Rydgel/monkey-rust/blob/master/lib/lexer/token.rs)
impl<'a> nom::InputIter for TokenStream<'a> {
    type Item = &'a Token;
    type Iter = std::iter::Enumerate<std::slice::Iter<'a, Token>>;
    type IterElem = std::slice::Iter<'a, Token>;

    #[inline]
    fn iter_indices(&self) -> std::iter::Enumerate<std::slice::Iter<'a, Token>> {
        self.tokens.iter().enumerate()
    }

    #[inline]
    fn iter_elements(&self) -> std::slice::Iter<'a, Token> {
        self.tokens.iter()
    }

    #[inline]
    fn position<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(Self::Item) -> bool,
    {
        self.tokens.iter().position(predicate)
    }

    #[inline]
    fn slice_index(&self, count: usize) -> Result<usize, nom::Needed> {
        if self.tokens.len() >= count {
            Ok(count)
        } else {
            Err(nom::Needed::Unknown)
        }
    }
}

impl std::fmt::Display for TokenType {
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

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.token_type)
    }
}
