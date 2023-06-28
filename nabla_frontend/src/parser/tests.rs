use crate::{ast::*, lexer::lex, parser::parse, token::TokenRange};
use pretty_assertions::assert_eq;

fn ident(name: &str, range: TokenRange) -> Ident {
    Ident {
        name: name.to_string(),
        info: AstInfo::new(range),
    }
}

fn info(range: TokenRange) -> AstInfo {
    AstInfo::new(range)
}

#[test]
fn empty() {
    let src = "";
    let tokens = lex(src);
    let program = parse(&tokens);
    assert_eq!(
        Program {
            globals: Vec::new(),
            info: AstInfo::new(0..1),
        },
        program
    );
}

#[test]
fn use_simple() {
    let src = "use a";
    let tokens = lex(src);
    let program = parse(&tokens);
    assert_eq!(
        Program {
            globals: vec![Global::Use(Use {
                use_kw: info(0..1),
                name: Some(ident("a", 1..3)),
                body: None,
                info: info(0..3),
            })],
            info: info(0..4),
        },
        program
    );
}

#[test]
fn use_all() {
    let src = "use a::*";
    let tokens = lex(src);
    let program = parse(&tokens);
    assert_eq!(
        Program {
            globals: vec![Global::Use(Use {
                use_kw: info(0..1),
                name: Some(ident("a", 1..3)),
                body: Some(UseBody {
                    double_colon: info(3..4),
                    kind: Some(UseKind::All(info(4..5))),
                    info: info(3..5),
                }),
                info: info(0..5),
            })],
            info: info(0..6),
        },
        program
    );
}

#[test]
fn use_single() {
    let src = "use a::b";
    let tokens = lex(src);
    let program = parse(&tokens);
    assert_eq!(
        Program {
            globals: vec![Global::Use(Use {
                use_kw: info(0..1),
                name: Some(ident("a", 1..3)),
                body: Some(UseBody {
                    double_colon: info(3..4),
                    kind: Some(UseKind::Single(UseItem {
                        name: ident("b", 4..5),
                        body: None,
                        alias: None,
                        info: info(4..5),
                    })),
                    info: info(3..5),
                }),
                info: info(0..5),
            })],
            info: info(0..6),
        },
        program
    );
}

#[test]
fn use_multiple() {
    let src = "use a::{b c}";
    let tokens = lex(src);
    let program = parse(&tokens);
    assert_eq!(
        Program {
            globals: vec![Global::Use(Use {
                use_kw: info(0..1),
                name: Some(ident("a", 1..3)),
                body: Some(UseBody {
                    double_colon: info(3..4),
                    kind: Some(UseKind::Multiple(UseItems {
                        lcurly: info(4..5),
                        names: vec![
                            UseItem {
                                name: ident("b", 5..6),
                                body: None,
                                alias: None,
                                info: info(5..6),
                            },
                            UseItem {
                                name: ident("c", 6..8),
                                body: None,
                                alias: None,
                                info: info(6..8),
                            },
                        ],
                        rcurly: Some(info(8..9)),
                        info: info(4..9),
                    })),
                    info: info(3..9),
                }),
                info: info(0..9),
            })],
            info: info(0..10),
        },
        program
    );
}

#[test]
fn use_complex() {
    let src = "use a::{b::{ c::d as x e::* } f as y}";
    let tokens = lex(src);
    let program = parse(&tokens);
    insta::assert_debug_snapshot!(program);
}

#[test]
fn def_ident() {
    let src = "def x = y";
    let tokens = lex(src);
    let program = parse(&tokens);
    assert_eq!(
        Program {
            globals: vec![Global::Def(Def {
                def_kw: info(0..1),
                name: Some(ident("x", 1..3)),
                eq: Some(info(3..5)),
                expr: Some(Expr::Single(Single::Named(Named {
                    name: ident("y", 5..7),
                    inner_names: Vec::new(),
                    expr: None,
                    info: info(5..7),
                }))),
                info: info(0..7),
            })],
            info: info(0..8),
        },
        program
    );
}

#[test]
fn def_union() {
    let src = "def ok = \"yes\" | true";
    let tokens = lex(src);
    let program = parse(&tokens);
    assert_eq!(
        Program {
            globals: vec![Global::Def(Def {
                def_kw: info(0..1),
                name: Some(ident("ok", 1..3)),
                eq: Some(info(3..5)),
                expr: Some(Expr::Union(Union {
                    single: Single::Primitive(Primitive::String(PrimiveValue {
                        value: "yes".to_string(),
                        info: info(5..7),
                    })),
                    alternatives: vec![UnionAlternative {
                        pipe: info(7..9),
                        single: Some(Single::Primitive(Primitive::Bool(info(9..11)))),
                        info: info(7..11),
                    }],
                    info: info(5..11),
                })),
                info: info(0..11),
            })],
            info: info(0..12),
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
    let tokens = lex(src);
    let program = parse(&tokens);
    assert_eq!(
        Program {
            globals: vec![Global::Def(Def {
                def_kw: info(0..2),
                name: Some(ident("Person", 2..4)),
                eq: Some(info(4..6)),
                expr: Some(Expr::Single(Single::Struct(Struct {
                    lcurly: info(6..8),
                    fields: vec![
                        StructField {
                            name: ident("name", 8..10),
                            colon: Some(info(10..11)),
                            type_expr: Some(Expr::Single(Single::Named(Named {
                                name: ident("string", 11..13),
                                inner_names: Vec::new(),
                                expr: None,
                                info: info(11..13),
                            }))),
                            eq: None,
                            expr: None,
                            alias: None,
                            info: info(8..13),
                        },
                        StructField {
                            name: ident("age", 13..15),
                            colon: Some(info(15..16)),
                            type_expr: Some(Expr::Single(Single::Named(Named {
                                name: ident("number", 16..18),
                                inner_names: Vec::new(),
                                expr: None,
                                info: info(16..18),
                            }))),
                            eq: Some(info(18..20)),
                            expr: Some(Expr::Single(Single::Primitive(Primitive::Number(
                                PrimiveValue {
                                    value: "0".to_string(),
                                    info: info(20..22),
                                }
                            )))),
                            alias: None,
                            info: info(13..22),
                        },
                    ],
                    rcurly: Some(info(22..24)),
                    info: info(6..24),
                }))),
                info: info(0..24),
            })],
            info: info(0..25),
        },
        program
    );
}

#[test]
fn def_list() {
    let src = "def Strings = [ string ]";
    let tokens = lex(src);
    let program = parse(&tokens);
    assert_eq!(
        Program {
            globals: vec![Global::Def(Def {
                def_kw: info(0..1),
                name: Some(ident("Strings", 1..3)),
                eq: Some(info(3..5)),
                expr: Some(Expr::Single(Single::List(List {
                    lbracket: info(5..7),
                    expr: Some(Box::new(Expr::Single(Single::Named(Named {
                        name: ident("string", 7..9),
                        inner_names: Vec::new(),
                        expr: None,
                        info: info(7..9),
                    })))),
                    rbracket: Some(info(9..11)),
                    info: info(5..11),
                }))),
                info: info(0..11),
            })],
            info: info(0..12),
        },
        program
    );
}

#[test]
fn def_all_syntax() {
    let src = "
use other_dir::*
use dir2::{x::{y} z}
use dir3::test

def x = {
    only_type: string
    only_expr = \"expr\"
    type_and_expr: number = 1
    only_type_as: string as \"x\"
    only_expr_as = \"expr\" as \"y\"
    type_and_expr_as: number = 1 as \"z\"
}
";
    let tokens = lex(src);
    let program = parse(&tokens);
    insta::assert_debug_snapshot!(program);
}
