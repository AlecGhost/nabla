use crate::{
    ast::*,
    lexer::lex,
    parser::{parse, Error, ErrorMessage},
    token::{self, Token, TokenRange, TokenType},
};
use pretty_assertions::assert_eq;

fn ident(name: &str, prelude_range: TokenRange, range: TokenRange) -> Ident {
    Ident {
        name: name.to_string(),
        info: AstInfo::new(Prelude::ranged(prelude_range), range),
    }
}

fn info(prelude_range: TokenRange, range: TokenRange) -> AstInfo {
    AstInfo::new(Prelude::ranged(prelude_range), range)
}

#[test]
fn empty() {
    let src = "";
    let (tokens, errors) = lex(src);
    assert_empty!(errors);
    let (program, errors) = parse(&tokens);
    assert_empty!(errors);
    assert_eq!(
        Program {
            globals: Vec::new(),
            info: info(0..0, 0..1),
        },
        program
    );
}

#[test]
fn token_after_eof() {
    let tokens = vec![
        Token::new(TokenType::Eof, 0..0),
        Token::new(TokenType::Eof, 0..0),
    ];
    let (_, errors) = parse(&tokens);
    assert_eq!(vec![Error::new(ErrorMessage::TokensAfterEof, 1..1)], errors);
}

#[test]
fn missing_type_expr() {
    let src = "let a: = {}";
    let (tokens, errors) = lex(src);
    assert_empty!(errors);
    let (_, errors) = parse(&tokens);
    assert_eq!(vec![Error::new(ErrorMessage::ExpectedExpr, 4..4)], errors);
}

#[test]
fn missing_expr() {
    let src = "let a =";
    let (tokens, errors) = lex(src);
    assert_empty!(errors);
    let (_, errors) = parse(&tokens);
    assert_eq!(vec![Error::new(ErrorMessage::ExpectedExpr, 4..4)], errors);
}

#[test]
fn missing_multiple_exprs() {
    let src = "let a: =";
    let (tokens, errors) = lex(src);
    assert_empty!(errors);
    let (_, errors) = parse(&tokens);
    assert_eq!(
        vec![
            Error::new(ErrorMessage::ExpectedExpr, 4..4),
            Error::new(ErrorMessage::ExpectedExpr, 5..5),
        ],
        errors
    );
}

#[test]
fn use_simple() {
    let src = "use a";
    let (tokens, errors) = lex(src);
    assert_empty!(errors);
    let (program, errors) = parse(&tokens);
    assert_empty!(errors);
    assert_eq!(
        Program {
            globals: vec![Global::Use(Use {
                use_kw: info(0..0, 0..1),
                name: Some(ident("a", 1..2, 2..3)),
                body: None,
                info: info(0..0, 0..3),
            })],
            info: info(0..0, 0..4),
        },
        program
    );
}

#[test]
fn use_all() {
    let src = "use a::*";
    let (tokens, errors) = lex(src);
    assert_empty!(errors);
    let (program, errors) = parse(&tokens);
    assert_empty!(errors);
    assert_eq!(
        Program {
            globals: vec![Global::Use(Use {
                use_kw: info(0..0, 0..1),
                name: Some(ident("a", 1..2, 2..3)),
                body: Some(UseBody {
                    double_colon: info(3..3, 3..4),
                    kind: Some(UseKind::All(info(4..4, 4..5))),
                    info: info(3..3, 3..5),
                }),
                info: info(0..0, 0..5),
            })],
            info: info(0..0, 0..6),
        },
        program
    );
}

#[test]
fn use_single() {
    let src = "use a::b";
    let (tokens, errors) = lex(src);
    assert_empty!(errors);
    let (program, errors) = parse(&tokens);
    assert_empty!(errors);
    assert_eq!(
        Program {
            globals: vec![Global::Use(Use {
                use_kw: info(0..0, 0..1),
                name: Some(ident("a", 1..1, 1..3)),
                body: Some(UseBody {
                    double_colon: info(3..3, 3..4),
                    kind: Some(UseKind::Single(UseItem {
                        name: ident("b", 4..4, 4..5),
                        body: None,
                        alias: None,
                        info: info(4..4, 4..5),
                    })),
                    info: info(3..3, 3..5),
                }),
                info: info(0..0, 0..5),
            })],
            info: info(0..0, 0..6),
        },
        program
    );
}

#[test]
fn use_multiple() {
    let src = "use a::{b c}";
    let (tokens, errors) = lex(src);
    assert_empty!(errors);
    let (program, errors) = parse(&tokens);
    assert_empty!(errors);
    assert_eq!(
        Program {
            globals: vec![Global::Use(Use {
                use_kw: info(0..0, 0..1),
                name: Some(ident("a", 1..2, 2..3)),
                body: Some(UseBody {
                    double_colon: info(3..3, 3..4),
                    kind: Some(UseKind::Multiple(UseItems {
                        lcurly: info(4..4, 4..5),
                        items: vec![
                            Ok(UseItem {
                                name: ident("b", 5..5, 5..6),
                                body: None,
                                alias: None,
                                info: info(5..5, 5..6),
                            }),
                            Ok(UseItem {
                                name: ident("c", 6..7, 7..8),
                                body: None,
                                alias: None,
                                info: info(6..7, 7..8),
                            }),
                        ],
                        rcurly: Some(info(8..8, 8..9)),
                        info: info(4..4, 4..9),
                    })),
                    info: info(3..3, 3..9),
                }),
                info: info(0..0, 0..9),
            })],
            info: info(0..0, 0..10),
        },
        program
    );
}

#[test]
fn use_complex() {
    let src = "use a::{b::{ c::d as x e::* } f as y}";
    let (tokens, errors) = lex(src);
    assert_empty!(errors);
    let (program, errors) = parse(&tokens);
    assert_empty!(errors);
    insta::assert_debug_snapshot!(program);
}

#[test]
fn def_ident() {
    let src = "def x = y";
    let (tokens, errors) = lex(src);
    assert_empty!(errors);
    let (program, errors) = parse(&tokens);
    assert_empty!(errors);
    assert_eq!(
        Program {
            globals: vec![Global::Def(Def {
                def_kw: info(0..0, 0..1),
                name: Some(ident("x", 1..2, 2..3)),
                colon: None,
                type_expr: None,
                eq: Some(info(3..4, 4..5)),
                expr: Some(Expr::Single(Single::Named(Named {
                    name: ident("y", 6..6, 6..7),
                    inner_names: Vec::new(),
                    expr: None,
                    info: info(6..6, 6..7),
                }))),
                info: info(0..0, 0..7),
            })],
            info: info(0..0, 0..8),
        },
        program
    );
}

#[test]
fn def_union() {
    let src = r#"def ok = "yes" | true"#;
    let (tokens, errors) = lex(src);
    assert_empty!(errors);
    let (program, errors) = parse(&tokens);
    assert_empty!(errors);
    assert_eq!(
        Program {
            globals: vec![Global::Def(Def {
                def_kw: info(0..0, 0..1),
                name: Some(ident("ok", 1..2, 2..3)),
                colon: None,
                type_expr: None,
                eq: Some(info(3..4, 4..5)),
                expr: Some(Expr::Union(Union {
                    single: Single::Primitive(Primitive::String(PrimitiveValue {
                        value: "yes".to_string(),
                        info: info(6..6, 6..7),
                    })),
                    alternatives: vec![UnionAlternative {
                        pipe: info(8..8, 8..9),
                        single: Some(Single::Primitive(Primitive::Bool(Bool::new_true(info(9..10,
                            10..11
                        ))))),
                        info: info(8..8, 8..11),
                    }],
                    info: info(5..6, 6..11),
                })),
                info: info(0..0, 0..11),
            })],
            info: info(0..0, 0..12),
        },
        program
    );
}

#[test]
fn def_struct() {
    let src = "
def Person = {
    name: string
    age: number = 0
}";
    let (tokens, errors) = lex(src);
    assert_empty!(errors);
    let (program, errors) = parse(&tokens);
    assert_empty!(errors);
    assert_eq!(
        Program {
            globals: vec![Global::Def(Def {
                def_kw: info(1..1, 1..2),
                name: Some(ident("Person", 2..3, 3..4)),
                colon: None,
                type_expr: None,
                eq: Some(info(4..5, 5..6)),
                expr: Some(Expr::Single(Single::Struct(Struct {
                    lcurly: info(7..7, 7..8),
                    fields: vec![
                        Ok(StructField {
                            name: ident("name", 9..9, 9..10),
                            colon: Some(info(10..10, 10..11)),
                            type_expr: Some(Expr::Single(Single::Named(Named {
                                name: ident("string", 12..12, 12..13),
                                inner_names: Vec::new(),
                                expr: None,
                                info: info(12..12, 12..13),
                            }))),
                            eq: None,
                            expr: None,
                            alias: None,
                            info: info(8..9, 9..14),
                        }),
                        Ok(StructField {
                            name: ident("age", 14..14, 14..15),
                            colon: Some(info(15..15, 15..16)),
                            type_expr: Some(Expr::Single(Single::Named(Named {
                                name: ident("number", 17..17, 17..18),
                                inner_names: Vec::new(),
                                expr: None,
                                info: info(17..17, 17..18),
                            }))),
                            eq: Some(info(19..19, 19..20)),
                            expr: Some(Expr::Single(Single::Primitive(Primitive::Number(
                                PrimitiveValue {
                                    value: "0".to_string(),
                                    info: info(21..21, 21..22),
                                }
                            )))),
                            alias: None,
                            info: info(14..14, 14..23),
                        }),
                    ],
                    rcurly: Some(info(23..23, 23..24)),
                    info: info(7..7, 7..24),
                }))),
                info: info(1..1, 1..24),
            })],
            info: info(0..1, 1..25),
        },
        program
    );
}

#[test]
fn def_list() {
    let src = "def Strings = [ string ]";
    let (tokens, errors) = lex(src);
    assert_empty!(errors);
    let (program, errors) = parse(&tokens);
    assert_empty!(errors);
    assert_eq!(
        Program {
            globals: vec![Global::Def(Def {
                def_kw: info(0..0, 0..1),
                name: Some(ident("Strings", 1..2, 2..3)),
                colon: None,
                type_expr: None,
                eq: Some(info(3..4, 4..5)),
                expr: Some(Expr::Single(Single::List(List {
                    lbracket: info(6..6, 6..7),
                    exprs: vec![Expr::Single(Single::Named(Named {
                        name: ident("string", 8..8, 8..9),
                        inner_names: Vec::new(),
                        expr: None,
                        info: info(8..8, 8..9),
                    }))],
                    rbracket: Some(info(10..10, 10..11)),
                    info: info(6..6, 6..11),
                }))),
                info: info(0..0, 0..11),
            })],
            info: info(0..0, 0..12),
        },
        program
    );
}

#[test]
fn def_all_syntax() {
    let src = r#"
use other_dir::*
use dir2::{x::{y} z}
use dir3::test

def x = {
    only_type: string
    only_expr = "expr"
    type_and_expr: number = 1
    only_type_as: string as "x"
    only_expr_as = "expr" as "y"
    type_and_expr_as: number = 1 as "z"
}
"#;
    let (tokens, errors) = lex(src);
    assert_empty!(errors);
    let (program, errors) = parse(&tokens);
    assert_empty!(errors);
    insta::assert_debug_snapshot!(program);
}

#[test]
fn ignore_expr_error() {
    let src = "def x = @";
    let (tokens, errors) = lex(src);
    assert_eq!(
        vec![token::Error::new(token::ErrorMessage::Unknown, 8..9)],
        errors
    );
    let (program, errors) = parse(&tokens);
    assert_eq!(
        vec![Error::new(ErrorMessage::UnexpectedTokens, 6..7)],
        errors
    );
    insta::assert_debug_snapshot!(program);
}

#[test]
fn ignore_global_error() {
    let src = "def x = {}=";
    let (tokens, errors) = lex(src);
    assert_empty!(errors);
    let (program, errors) = parse(&tokens);
    assert_eq!(
        vec![Error::new(ErrorMessage::UnexpectedTokens, 8..9)],
        errors
    );
    insta::assert_debug_snapshot!(program);
}

#[test]
fn ignore_use_kind_error() {
    let src = "use x::{y::=}";
    let (tokens, errors) = lex(src);
    assert_empty!(errors);
    let (program, errors) = parse(&tokens);
    assert_eq!(
        vec![Error::new(ErrorMessage::UnexpectedTokens, 7..8)],
        errors
    );
    insta::assert_debug_snapshot!(program);
}

#[test]
fn ignore_use_item_error() {
    let src = "use x::{=}";
    let (tokens, errors) = lex(src);
    assert_empty!(errors);
    let (program, errors) = parse(&tokens);
    assert_eq!(
        vec![Error::new(ErrorMessage::UnexpectedTokens, 5..6)],
        errors
    );
    insta::assert_debug_snapshot!(program);
}

#[test]
fn comma_after_field() {
    let src = "
def x = {
    test: String,
}
";
    let (tokens, errors) = lex(src);
    assert_eq!(
        vec![token::Error::new(token::ErrorMessage::Unknown, 27..28)],
        errors
    );
    let (program, errors) = parse(&tokens);
    assert_eq!(
        vec![Error::new(ErrorMessage::UnexpectedTokens, 13..14)],
        errors
    );
    insta::assert_debug_snapshot!(program);
}
