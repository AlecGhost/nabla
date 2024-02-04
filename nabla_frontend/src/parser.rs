use crate::{
    ast::*,
    token::{Token, TokenStream, TokenType},
};
pub use error::*;
use nom::{
    branch::alt,
    combinator::{map, opt},
    multi::many0,
    sequence::tuple,
};

use self::utility::{expect, ignore_until, info};

mod error;
#[cfg(test)]
mod tests;
mod utility;

type IResult<'a, T> = nom::IResult<TokenStream<'a>, T, ParserError<'a>>;

/// Parses the given tokens and returns an AST.
///
/// # Panics
///
/// Panics if parsing fails.
pub fn parse(input: &[Token]) -> (Ast, Vec<Error>) {
    let (mut token_stream, ast) = Ast::parse(input.into()).expect("Parser cannot fail");
    if !token_stream.tokens().is_empty() {
        let offset = token_stream.location_offset();
        token_stream.append_error(Error::new(ErrorMessage::TokensAfterEof, offset..offset))
    }
    let errors = token_stream.error_buffer;
    (ast, errors)
}

trait Parser: Sized {
    fn parse(input: TokenStream) -> IResult<Self>;
}

impl Parser for Ast {
    fn parse(input: TokenStream) -> IResult<Self> {
        map(
            info(tuple((many0(Global::parse), token::eof))),
            |((globals, _), info)| Self { globals, info },
        )(input)
    }
}

impl Parser for Global {
    fn parse(input: TokenStream) -> IResult<Self> {
        alt((
            map(Use::parse, Self::Use),
            map(Def::parse, Self::Def),
            map(Let::parse, Self::Let),
            map(Expr::parse, Self::Init),
            map(info(ignore_until(lookahead::global)), |(_, info)| {
                Self::Error(info)
            }),
        ))(input)
    }
}

impl Parser for Use {
    fn parse(input: TokenStream) -> IResult<Self> {
        map(
            info(tuple((
                token::r#use,
                expect(Ident::parse, ErrorMessage::ExpectedIdent),
                opt(UseBody::parse),
            ))),
            |((use_kw, name, body), info)| Self {
                use_kw,
                name,
                body,
                info,
            },
        )(input)
    }
}

impl Parser for UseBody {
    fn parse(input: TokenStream) -> IResult<Self> {
        map(
            info(tuple((
                token::double_colon,
                expect(UseKind::parse, ErrorMessage::ExpectedUseKind),
            ))),
            |((double_colon, kind), info)| Self {
                double_colon,
                kind,
                info,
            },
        )(input)
    }
}

impl Parser for UseKind {
    fn parse(input: TokenStream) -> IResult<Self> {
        alt((
            map(token::star, Self::All),
            map(UseItem::parse, Self::Single),
            map(UseItems::parse, Self::Multiple),
            map(info(ignore_until(lookahead::use_kind)), |(_, info)| {
                Self::Error(info)
            }),
        ))(input)
    }
}

impl Parser for UseItems {
    fn parse(input: TokenStream) -> IResult<Self> {
        map(
            info(tuple((
                token::lcurly,
                many0(alt((
                    map(UseItem::parse, Ok),
                    map(UseItemError::parse, Err),
                ))),
                expect(
                    map(info(token::rcurly), |(_, info)| info),
                    ErrorMessage::MissingClosingCurly,
                ),
            ))),
            |((lcurly, names, rcurly), info)| Self {
                lcurly,
                items: names,
                rcurly,
                info,
            },
        )(input)
    }
}

impl Parser for UseItem {
    fn parse(input: TokenStream) -> IResult<Self> {
        map(
            info(tuple((
                Ident::parse,
                opt(UseBody::parse),
                opt(Alias::parse),
            ))),
            |((name, body, alias), info)| Self {
                name,
                body: body.map(Box::new),
                alias,
                info,
            },
        )(input)
    }
}

impl Parser for UseItemError {
    fn parse(input: TokenStream) -> IResult<Self> {
        map(info(ignore_until(lookahead::use_item)), |(_, info)| Self {
            info,
        })(input)
    }
}

impl Parser for Def {
    fn parse(input: TokenStream) -> IResult<Self> {
        map(
            info(tuple((
                token::def,
                expect(Ident::parse, ErrorMessage::ExpectedIdent),
                parse_type_annotation,
                expect(token::eq, ErrorMessage::ExpectedEQ),
                expect(Expr::parse, ErrorMessage::ExpectedExpr),
            ))),
            |((def_kw, name, (colon, type_expr), eq, expr), info)| Self {
                def_kw,
                name,
                colon,
                type_expr,
                eq,
                expr,
                info,
            },
        )(input)
    }
}

impl Parser for Let {
    fn parse(input: TokenStream) -> IResult<Self> {
        map(
            info(tuple((
                token::r#let,
                expect(Ident::parse, ErrorMessage::ExpectedIdent),
                parse_type_annotation,
                expect(token::eq, ErrorMessage::ExpectedEQ),
                expect(Expr::parse, ErrorMessage::ExpectedExpr),
            ))),
            |((let_kw, name, (colon, type_expr), eq, expr), info)| Self {
                let_kw,
                name,
                colon,
                type_expr,
                eq,
                expr,
                info,
            },
        )(input)
    }
}

impl Parser for Expr {
    fn parse(input: TokenStream) -> IResult<Self> {
        match info(Single::parse)(input) {
            Ok((input, (single, single_info))) => {
                let (input, (alternatives, alt_info)) =
                    info(many0(UnionAlternative::parse))(input)?;
                if alternatives.is_empty() {
                    Ok((input, Self::Single(single)))
                } else {
                    let start = single_info.range.start;
                    let end = alt_info.range.end;
                    Ok((
                        input,
                        Self::Union(Union {
                            single,
                            alternatives,
                            info: AstInfo::new(single_info.prelude, start..end),
                        }),
                    ))
                }
            }
            Err(nom::Err::Error(err)) => map(info(ignore_until(lookahead::expr)), |(_, info)| {
                Self::Error(info)
            })(err.input),
            Err(_) => panic!("Expr: unexpected error"),
        }
    }
}

impl Parser for UnionAlternative {
    fn parse(input: TokenStream) -> IResult<Self> {
        map(
            info(tuple((
                token::pipe,
                expect(Single::parse, ErrorMessage::ExpectedSingle),
            ))),
            |((pipe, single), info)| Self { pipe, single, info },
        )(input)
    }
}

impl Parser for Single {
    fn parse(input: TokenStream) -> IResult<Self> {
        alt((
            map(Struct::parse, Self::Struct),
            map(List::parse, Self::List),
            map(Named::parse, Self::Named),
            map(Primitive::parse, Self::Primitive),
        ))(input)
    }
}

impl Parser for Struct {
    fn parse(input: TokenStream) -> IResult<Self> {
        map(
            info(tuple((
                token::lcurly,
                many0(alt((
                    map(StructField::parse, Ok),
                    map(StructFieldError::parse, Err),
                ))),
                expect(token::rcurly, ErrorMessage::MissingClosingCurly),
            ))),
            |((lcurly, fields, rcurly), info)| Self {
                lcurly,
                fields,
                rcurly,
                info,
            },
        )(input)
    }
}

impl Parser for StructField {
    fn parse(input: TokenStream) -> IResult<Self> {
        map(
            info(tuple((
                Ident::parse,
                parse_type_annotation,
                map(
                    opt(tuple((
                        token::eq,
                        expect(Expr::parse, ErrorMessage::ExpectedExpr),
                    ))),
                    |opt| match opt {
                        Some((eq, expr)) => (Some(eq), expr),
                        None => (None, None),
                    },
                ),
                opt(Alias::parse),
            ))),
            |((name, (colon, type_expr), (eq, expr), alias), info)| Self {
                name,
                colon,
                type_expr,
                eq,
                expr,
                alias,
                info,
            },
        )(input)
    }
}

impl Parser for StructFieldError {
    fn parse(input: TokenStream) -> IResult<Self> {
        map(info(ignore_until(lookahead::struct_field)), |(_, info)| {
            Self { info }
        })(input)
    }
}

impl Parser for List {
    fn parse(input: TokenStream) -> IResult<Self> {
        map(
            info(tuple((
                token::lbracket,
                many0(Expr::parse),
                expect(token::rbracket, ErrorMessage::MissingClosingBracket),
            ))),
            |((lbracket, exprs, rbracket), info)| Self {
                lbracket,
                exprs,
                rbracket,
                info,
            },
        )(input)
    }
}

impl Parser for Named {
    fn parse(input: TokenStream) -> IResult<Self> {
        map(
            info(tuple((
                Ident::parse,
                many0(InnerName::parse),
                opt(alt((
                    map(Struct::parse, StructOrList::Struct),
                    map(List::parse, StructOrList::List),
                ))),
            ))),
            |((name, inner_names, expr), info)| Self {
                name,
                inner_names,
                expr,
                info,
            },
        )(input)
    }
}

impl Parser for InnerName {
    fn parse(input: TokenStream) -> IResult<Self> {
        map(
            info(tuple((
                token::double_colon,
                expect(Ident::parse, ErrorMessage::ExpectedIdent),
            ))),
            |((double_colon, name), info)| Self {
                double_colon,
                name,
                info,
            },
        )(input)
    }
}

impl Parser for Primitive {
    fn parse(input: TokenStream) -> IResult<Self> {
        alt((
            map(map(token::string, PrimitiveValue::new), Self::String),
            map(map(token::char, PrimitiveValue::new), Self::Char),
            map(map(token::number, PrimitiveValue::new), Self::Number),
            map(Bool::parse, Self::Bool),
            map(token::null, Self::Null),
        ))(input)
    }
}

impl Parser for Alias {
    fn parse(input: TokenStream) -> IResult<Self> {
        map(
            info(tuple((
                token::r#as,
                opt(AliasName::parse), // expect message is injected later
            ))),
            |((as_kw, name), info)| Self { as_kw, name, info },
        )(input)
    }
}

impl Parser for AliasName {
    fn parse(input: TokenStream) -> IResult<Self> {
        alt((
            map(map(token::string, PrimitiveValue::new), Self::String),
            map(Ident::parse, Self::Ident),
        ))(input)
    }
}

impl Parser for Ident {
    fn parse(input: TokenStream) -> IResult<Self> {
        map(token::ident, |(name, info)| Self { name, info })(input)
    }
}

impl Parser for Bool {
    fn parse(input: TokenStream) -> IResult<Self> {
        alt((
            map(token::r#true, Self::new_true),
            map(token::r#false, Self::new_false),
        ))(input)
    }
}

impl Parser for Prelude {
    fn parse(input: TokenStream) -> IResult<Self> {
        let start = input.location_offset();
        let (input, tokens) = many0(alt((token::whitespace, token::comment)))(input)?;
        let end = input.location_offset();
        let comments = tokens
            .iter()
            .filter_map(|token| {
                if let TokenType::Comment(comment) = &token.token_type {
                    Some(comment.clone())
                } else {
                    None
                }
            })
            .collect();
        Ok((
            input,
            Self {
                comments,
                range: start..end,
            },
        ))
    }
}

fn parse_type_annotation(input: TokenStream) -> IResult<(Option<AstInfo>, Option<Expr>)> {
    map(
        opt(tuple((
            token::colon,
            expect(Expr::parse, ErrorMessage::ExpectedExpr),
        ))),
        |opt| match opt {
            Some((colon, type_expr)) => (Some(colon), type_expr),
            None => (None, None),
        },
    )(input)
}

mod token {
    use crate::{
        ast::AstInfo,
        parser::{utility::info, IResult, ParserError, ParserErrorKind},
        token::{Token, TokenStream, TokenType},
    };
    use nom::bytes::complete::take;

    pub fn comment(input: TokenStream) -> IResult<Token> {
        let original_input = input.clone();
        let (input, token_stream) = take(1usize)(input)?;
        let token = token_stream
            .first_token()
            .cloned()
            .expect("TokenStream must no be empty");
        if matches!(token.token_type, TokenType::Comment(_)) {
            Ok((input, token))
        } else {
            Err(nom::Err::Error(ParserError {
                kind: ParserErrorKind::Token,
                input: original_input,
            }))
        }
    }

    pub fn whitespace(input: TokenStream) -> IResult<Token> {
        let original_input = input.clone();
        let (input, token_stream) = take(1usize)(input)?;
        let token = token_stream
            .first_token()
            .cloned()
            .expect("TokenStream must no be empty");
        if matches!(token.token_type, TokenType::Whitespace(_)) {
            Ok((input, token))
        } else {
            Err(nom::Err::Error(ParserError {
                kind: ParserErrorKind::Token,
                input: original_input,
            }))
        }
    }

    macro_rules! simple_token_parser {
        ($name:ident, $token_type:pat) => {
            pub fn $name(input: TokenStream) -> IResult<AstInfo> {
                let original_input = input.clone();
                let (input, (token_stream, info)) = info(take(1usize))(input)?;
                let token = token_stream
                    .first_token()
                    .cloned()
                    .expect("TokenStream must no be empty");
                if matches!(token.token_type, $token_type) {
                    Ok((input, info))
                } else {
                    Err(nom::Err::Error(ParserError {
                        kind: ParserErrorKind::Token,
                        input: original_input,
                    }))
                }
            }
        };
    }

    macro_rules! complex_token_parser {
        ($name:ident, $token_type:path) => {
            pub fn $name(input: TokenStream) -> IResult<(String, AstInfo)> {
                let original_input = input.clone();
                let (input, (token_stream, info)) = info(take(1usize))(input)?;
                let token = token_stream
                    .first_token()
                    .cloned()
                    .expect("TokenStream must no be empty");
                match token.token_type {
                    $token_type(s) => Ok((input, (s, info))),
                    _ => Err(nom::Err::Error(ParserError {
                        kind: ParserErrorKind::Token,
                        input: original_input,
                    })),
                }
            }
        };
    }

    // Tokens that are self-defined
    simple_token_parser!(lbracket, TokenType::LBracket);
    simple_token_parser!(rbracket, TokenType::RBracket);
    simple_token_parser!(lcurly, TokenType::LCurly);
    simple_token_parser!(rcurly, TokenType::RCurly);
    simple_token_parser!(double_colon, TokenType::DoubleColon);
    simple_token_parser!(star, TokenType::Star);
    simple_token_parser!(pipe, TokenType::Pipe);
    simple_token_parser!(eq, TokenType::Eq);
    simple_token_parser!(colon, TokenType::Colon);
    simple_token_parser!(r#use, TokenType::Use);
    simple_token_parser!(def, TokenType::Def);
    simple_token_parser!(r#let, TokenType::Let);
    simple_token_parser!(r#as, TokenType::As);
    simple_token_parser!(r#true, TokenType::True);
    simple_token_parser!(r#false, TokenType::False);
    simple_token_parser!(null, TokenType::Null);
    simple_token_parser!(eof, TokenType::Eof);

    // Tokens with string inside.
    complex_token_parser!(string, TokenType::String);
    complex_token_parser!(char, TokenType::Char);
    complex_token_parser!(number, TokenType::Number);
    complex_token_parser!(ident, TokenType::Ident);
}

mod lookahead {
    use crate::{
        parser::{token, IResult},
        token::TokenStream,
    };
    use nom::{
        branch::alt,
        combinator::{peek, recognize},
    };

    macro_rules! lookahead_parser {
        ($name:ident, $($parser:expr,)+) => {
            pub fn $name(input: TokenStream) -> IResult<TokenStream> {
                peek(alt((
                    $(recognize($parser),)+
                )))(input)
            }
        };
    }

    lookahead_parser!(
        global,
        token::r#use,
        token::def,
        token::r#let,
        token::lcurly,
        token::lbracket,
        token::ident,
        token::eof,
    );
    lookahead_parser!(use_kind, token::star, token::r#as, token::rcurly, global,);
    lookahead_parser!(use_item, use_kind,);
    lookahead_parser!(
        expr,
        token::eq,
        token::rcurly,
        token::rbracket,
        token::r#as,
        global,
    );
    lookahead_parser!(struct_field, token::pipe, expr,);
}
