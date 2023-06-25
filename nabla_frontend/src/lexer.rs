use crate::token::{Error, ErrorMessage, TextRange, ToTextRange, Token, TokenType};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_till},
    character::complete::{anychar, digit1, multispace1},
    combinator::{eof, map, opt, peek, recognize, verify},
    multi::many0,
    sequence::{delimited, pair, preceded, terminated, tuple},
};
use utility::{alpha_numeric1, expect, is_alpha_numeric};

#[cfg(test)]
mod tests;
mod utility;

/// Type alias for nom_locate::LocatedSpan.
/// Tracks range inside source code during lexical analysis.
type Span<'a> = nom_locate::LocatedSpan<&'a str>;

impl ToTextRange for Span<'_> {
    fn to_text_range(&self) -> TextRange {
        let start = self.location_offset();
        let end = start + self.fragment().len();
        start..end
    }
}

type IResult<'a> = nom::IResult<Span<'a>, Token>;

/// Tokenizes the given source code.
///
/// # Panics
///
/// Panics if lexing fails.
pub fn lex(src: &str) -> Vec<Token> {
    let input = Span::new(src);
    let (_, (mut tokens, eof_token)) =
        pair(many0(Token::lex), Eof::lex)(input).expect("Lexing must not fail.");
    tokens.push(eof_token);
    tokens
}

/// Try to parse `Span` into `Token`
trait Lexer: Sized {
    fn lex(input: Span) -> IResult;
}

/// Recognize symbol and map to `Token`
macro_rules! lex_symbol {
    ($token_type:expr) => {{
        map(
            tag($token_type
                .as_static_str()
                .expect("Static representation must be available")),
            |span: Span| -> Token { Token::new($token_type, span.to_text_range()) },
        )
    }};
}

/// Recognize keyword and map to `Token`
/// Same as `lex_symbol`, except that the next character must not be alpha_numeric
macro_rules! lex_keyword {
    ($token_type:expr) => {{
        terminated(
            map(
                tag($token_type
                    .as_static_str()
                    .expect("No static representation available")),
                |span: Span| -> Token { Token::new($token_type, span.to_text_range()) },
            ),
            peek(alt((
                recognize(eof),
                recognize(verify(anychar, |c| !is_alpha_numeric(*c))),
            ))),
        )
    }};
}

impl Lexer for Token {
    fn lex(input: Span) -> IResult {
        alt((
            lex_symbol!(TokenType::LBracket),
            lex_symbol!(TokenType::RBracket),
            lex_symbol!(TokenType::LCurly),
            lex_symbol!(TokenType::RCurly),
            lex_symbol!(TokenType::DoubleColon),
            lex_symbol!(TokenType::Star),
            lex_symbol!(TokenType::Pipe),
            lex_symbol!(TokenType::Eq),
            lex_symbol!(TokenType::Colon),
            lex_keyword!(TokenType::Use),
            lex_keyword!(TokenType::Def),
            lex_keyword!(TokenType::Let),
            lex_keyword!(TokenType::As),
            lex_keyword!(TokenType::True),
            lex_keyword!(TokenType::False),
            alt((
                String::lex,
                Char::lex,
                Number::lex,
                Ident::lex,
                Comment::lex,
                Whitespace::lex,
                Unknown::lex,
            )),
        ))(input)
    }
}

impl Lexer for String {
    fn lex(input: Span) -> IResult {
        let start = input.location_offset();
        let (input, s) = delimited(tag("\""), take_till(|c| c == '\"'), tag("\""))(input)?;
        let end = input.location_offset();
        Ok((
            input,
            Token::new(TokenType::String(s.to_string()), start..end),
        ))
    }
}

struct Char;
impl Lexer for Char {
    fn lex(input: Span) -> IResult {
        let start = input.location_offset();
        let (input, (_, char, closing_quote)) = tuple((
            tag("'"),
            alt((
                map(preceded(tag("\\"), anychar), |c| {
                    "\\".to_string() + &c.to_string()
                }),
                map(peek(tag("'")), |_| "".to_string()),
                map(anychar, |c| c.to_string()),
            )),
            expect(tag("'"), ErrorMessage::MissingClosingSingleQuote),
        ))(input)?;
        let end = input.location_offset();
        let mut token = Token::new(TokenType::Char(char), start..end);
        if let Err(quote_err) = closing_quote {
            token = token.append_error(quote_err);
        }
        Ok((input, token))
    }
}

struct Number;
impl Lexer for Number {
    fn lex(input: Span) -> IResult {
        let start = input.location_offset();
        let (input, (pre_decimals, decimals)) = pair(
            digit1,
            opt(preceded(
                tag("."),
                expect(digit1, ErrorMessage::MissingDecimals),
            )),
        )(input)?;
        let end = input.location_offset();
        let range = start..end;
        let token = match decimals {
            Some(Ok(decimals)) => Token::new(
                TokenType::Number(pre_decimals.to_string() + "." + decimals.fragment()),
                range,
            ),
            Some(Err(decimal_err)) => {
                Token::new(TokenType::Number(pre_decimals.to_string() + "."), range)
                    .append_error(decimal_err)
            }
            None => Token::new(TokenType::Number(pre_decimals.to_string()), range),
        };
        Ok((input, token))
    }
}

struct Ident;
impl Lexer for Ident {
    fn lex(input: Span) -> IResult {
        let start = input.location_offset();
        let (input, ident) = alpha_numeric1(input)?;
        let end = input.location_offset();
        Ok((
            input,
            Token::new(TokenType::Ident(ident.to_string()), start..end),
        ))
    }
}

struct Whitespace;
impl Lexer for Whitespace {
    fn lex(input: Span) -> IResult {
        let (input, whitespace) = multispace1(input)?;
        Ok((
            input,
            Token::new(
                TokenType::Whitespace(whitespace.to_string()),
                whitespace.to_text_range(),
            ),
        ))
    }
}

struct Comment;
impl Lexer for Comment {
    fn lex(input: Span) -> IResult {
        let start = input.location_offset();
        let (input, text) = delimited(tag("//"), take_till(|c| c == '\n'), tag("\n"))(input)?;
        let comment = "//".to_string() + text.fragment() + "\n";
        let end = input.location_offset();
        Ok((input, Token::new(TokenType::Comment(comment), start..end)))
    }
}

struct Unknown;
impl Lexer for Unknown {
    fn lex(input: Span) -> IResult {
        let start = input.location_offset();
        let (input, char) = anychar(input)?;
        let end = input.location_offset();
        let token = Token::new(TokenType::Unknown(char.to_string()), start..end)
            .append_error(Error::new(ErrorMessage::Unknown, start..end));
        Ok((input, token))
    }
}

struct Eof;
impl Lexer for Eof {
    fn lex(input: Span) -> IResult {
        map(eof, |span: Span| {
            Token::new(TokenType::Eof, span.to_text_range())
        })(input)
    }
}
