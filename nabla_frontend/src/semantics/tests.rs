use crate::{
    ast::Global,
    eval::{eval, Value},
    lexer::lex,
    parser::parse,
    semantics::{
        error::{Error, ErrorMessage},
        types::{self, TypeInfo},
    },
};
use pretty_assertions::assert_eq;

#[test]
fn empty() {
    let src = "";
    let (tokens, errors) = lex(src);
    assert_empty!(errors);
    let (program, errors) = parse(&tokens);
    assert_empty!(errors);
    let TypeInfo { errors, .. } = types::analyze(&program);
    assert_empty!(errors);
}

#[test]
fn no_reimport() {
    let src = "
use a
use b::{c d::e}
use f::g as h
";
    let (tokens, errors) = lex(src);
    assert_empty!(errors);
    let (program, errors) = parse(&tokens);
    assert_empty!(errors);
    let TypeInfo { errors, .. } = types::analyze(&program);
    assert_empty!(errors);
}

#[test]
fn reimport() {
    let src = "
use a::b
use c::b
";
    let (tokens, errors) = lex(src);
    assert_empty!(errors);
    let (program, errors) = parse(&tokens);
    assert_empty!(errors);
    let TypeInfo { errors, .. } = types::analyze(&program);
    assert_eq!(
        vec![Error::new(
            ErrorMessage::Redeclaration("b".to_string()),
            11..12
        )],
        errors
    );
}

#[test]
fn no_reimport_alias() {
    let src = "
use a::b
use c::b as d
";
    let (tokens, errors) = lex(src);
    assert_empty!(errors);
    let (program, errors) = parse(&tokens);
    assert_empty!(errors);
    let TypeInfo { errors, .. } = types::analyze(&program);
    assert_empty!(errors);
}

#[test]
fn empty_list() {
    let src = "
def EmptyList = []
EmptyList []
";
    let (tokens, errors) = lex(src);
    assert_empty!(errors);
    let (program, errors) = parse(&tokens);
    assert_empty!(errors);
    let TypeInfo { errors, .. } = types::analyze(&program);
    assert_empty!(errors);
}

#[test]
fn struct_fields() {
    let src = r#"
def Person = {
    name: String
    age: Number
}
Person {
    name: "Test"
    age: 0
}
"#;
    let (tokens, errors) = lex(src);
    assert_empty!(errors);
    let (program, errors) = parse(&tokens);
    assert_empty!(errors);
    let TypeInfo { errors, .. } = types::analyze(&program);
    assert_empty!(errors);
}

#[test]
fn optional() {
    let src = "
def Optional = Number | null
let opt_none: Optional = null
let opt_some: Optional = 1
";
    let (tokens, errors) = lex(src);
    assert_empty!(errors);
    let (program, errors) = parse(&tokens);
    assert_empty!(errors);
    let TypeInfo { errors, .. } = super::analyze(&program);
    assert_empty!(errors);
}

#[test]
fn evaluate_struct() {
    let src = r#"
{
    name = "Test"
    age = 0
    const: "x"  = "x"
}
"#;
    let (tokens, errors) = lex(src);
    assert_empty!(errors);
    let (program, errors) = parse(&tokens);
    assert_empty!(errors);
    let TypeInfo { errors, .. } = super::analyze(&program);
    assert_empty!(errors);
    let init = program
        .globals
        .iter()
        .find_map(|global| match global {
            Global::Init(init) => Some(init),
            _ => None,
        })
        .unwrap();
    let value = eval(init);
    assert_eq!(
        Value::from([
            ("name", Value::from("Test")),
            ("age", Value::from(0)),
            ("const", Value::from("x"))
        ]),
        value
    );
}

#[test]
fn evaluate_list() {
    let src = r#"["a" "b" "c"]"#;
    let (tokens, errors) = lex(src);
    assert_empty!(errors);
    let (program, errors) = parse(&tokens);
    assert_empty!(errors);
    let TypeInfo { errors, .. } = super::analyze(&program);
    assert_empty!(errors);
    let init = program
        .globals
        .iter()
        .find_map(|global| match global {
            Global::Init(init) => Some(init),
            _ => None,
        })
        .unwrap();
    let value = eval(init);
    assert_eq!(Value::from(["a", "b", "c"]), value);
}

#[test]
fn evaluate_complex_struct() {
    let src = r#"
{
    random_number = 42
    primes = [1 2 3 5 7]
    map = [
        {
            key = "a"
            value = "1"
        }
        {
            key = "b"
            value = null
        }
        {
            key = "c"
            value = true
        }
    ]
}
"#;
    let (tokens, errors) = lex(src);
    assert_empty!(errors);
    let (program, errors) = parse(&tokens);
    assert_empty!(errors);
    let TypeInfo { errors, .. } = super::analyze(&program);
    assert_empty!(errors);
    let init = program
        .globals
        .iter()
        .find_map(|global| match global {
            Global::Init(init) => Some(init),
            _ => None,
        })
        .unwrap();
    let value = eval(init);
    assert_eq!(
        Value::from([
            ("random_number", Value::from(42)),
            ("primes", Value::from([1, 2, 3, 5, 7])),
            (
                "map",
                Value::from([
                    Value::from([("key", Value::from("a")), ("value", Value::from("1")),]),
                    Value::from([("key", Value::from("b")), ("value", Value::Null),]),
                    Value::from([("key", Value::from("c")), ("value", Value::from(true)),]),
                ])
            )
        ]),
        value
    );
}

#[test]
fn built_in_type_equality() {
    let src = r#"
def Config = {
    version: String
}
Config {
    version: String = "1.0.0"
}
"#;
    let (tokens, errors) = lex(src);
    assert_empty!(errors);
    let (program, errors) = parse(&tokens);
    assert_empty!(errors);
    let TypeInfo { errors, .. } = super::analyze(&program);
    assert_empty!(errors);
}

#[test]
fn self_reference_expr() {
    let src = "
def Type = Type {}
";
    let (tokens, errors) = lex(src);
    assert_empty!(errors);
    let (program, errors) = parse(&tokens);
    assert_empty!(errors);
    let TypeInfo { errors, .. } = super::analyze(&program);
    assert_eq!(
        vec![Error::new(
            ErrorMessage::SelfReference("Type".to_string()),
            3..4
        )],
        errors
    );
}

#[test]
fn self_reference_type_expr() {
    let src = "
def Type: Type = {}
";
    let (tokens, errors) = lex(src);
    assert_empty!(errors);
    let (program, errors) = parse(&tokens);
    assert_empty!(errors);
    let TypeInfo { errors, .. } = super::analyze(&program);
    assert_eq!(
        vec![Error::new(
            ErrorMessage::SelfReference("Type".to_string()),
            3..4
        )],
        errors
    );
}

#[test]
fn legal_self_reference() {
    let src = r#"
def Type = [ Type | String ]
Type [ "a" [ "b" ] ]
"#;
    let (tokens, errors) = lex(src);
    assert_empty!(errors);
    let (program, errors) = parse(&tokens);
    assert_empty!(errors);
    let TypeInfo { errors, .. } = super::analyze(&program);
    assert_empty!(errors);
}

#[test]
fn type_annotation_in_init() {
    let src = r#"
def A = {
    a: String | null
}
A {
    a: String | null = null
}
"#;
    let (tokens, errors) = lex(src);
    assert_empty!(errors);
    let (program, errors) = parse(&tokens);
    assert_empty!(errors);
    let TypeInfo { errors, .. } = super::analyze(&program);
    assert_empty!(errors);
}

#[test]
fn type_annotation_subset() {
    let src = r#"
def A = {
    a: String | Number | null
}
A {
    a: String | null = null
}
"#;
    let (tokens, errors) = lex(src);
    assert_empty!(errors);
    let (program, errors) = parse(&tokens);
    assert_empty!(errors);
    let TypeInfo { errors, .. } = super::analyze(&program);
    assert_empty!(errors);
}

#[test]
fn type_annotation_superset() {
    let src = r#"
def A = {
    a: String | null
}
A {
    a: String | Number | null = null
}
"#;
    let (tokens, errors) = lex(src);
    assert_empty!(errors);
    let (program, errors) = parse(&tokens);
    assert_empty!(errors);
    let TypeInfo { errors, .. } = super::analyze(&program);
    assert_eq!(vec![Error::new(ErrorMessage::TypeMismatch, 31..32)], errors);
}

#[test]
fn union_in_let() {
    let src = r#"
let a: String = "A" | "a"
let b = "B" | "b"
"#;
    let (tokens, errors) = lex(src);
    assert_empty!(errors);
    let (program, errors) = parse(&tokens);
    assert_empty!(errors);
    let TypeInfo { errors, .. } = super::analyze(&program);
    assert_eq!(
        vec![
            Error::new(ErrorMessage::UnionInInit, 9..15),
            Error::new(ErrorMessage::UnionInInit, 21..27),
        ],
        errors
    );
}

#[test]
fn union_in_field() {
    let src = r#"
def Test = {
    a: String = "A" | "a"
    b = "B" | "b"
}
"#;
    let (tokens, errors) = lex(src);
    assert_empty!(errors);
    let (program, errors) = parse(&tokens);
    assert_empty!(errors);
    let TypeInfo { errors, .. } = super::analyze(&program);
    assert_eq!(
        vec![
            Error::new(ErrorMessage::UnionInInit, 15..21),
            Error::new(ErrorMessage::UnionInInit, 25..31),
        ],
        errors
    );
}
