use crate::{
    ast::Global,
    eval::{eval, Value},
    lexer::lex,
    parser::parse,
    semantics::{
        error::{Error, ErrorMessage},
        types::analyze,
    },
};
use pretty_assertions::assert_eq;

#[test]
fn empty() {
    let src = "";
    let tokens = lex(src);
    let (program, errors) = parse(&tokens);
    assert!(errors.is_empty());
    let errors = analyze(&program);
    assert!(errors.is_empty());
}

#[test]
fn no_reimport() {
    let src = "
use a
use b::{c d::e}
use f::g as h
";
    let tokens = lex(src);
    let (program, errors) = parse(&tokens);
    assert!(errors.is_empty());
    let errors = analyze(&program);
    assert!(errors.is_empty());
}

#[test]
fn reimport() {
    let src = "
use a::b
use c::b
";
    let tokens = lex(src);
    let (program, errors) = parse(&tokens);
    assert!(errors.is_empty());
    let errors = analyze(&program);
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
    let tokens = lex(src);
    let (program, errors) = parse(&tokens);
    assert!(errors.is_empty());
    let errors = analyze(&program);
    assert!(errors.is_empty());
}

#[test]
fn empty_list() {
    let src = "
def EmptyList = []
EmptyList []
";
    let tokens = lex(src);
    let (program, errors) = parse(&tokens);
    assert!(errors.is_empty());
    let errors = analyze(&program);
    assert!(errors.is_empty());
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
    let tokens = lex(src);
    let (program, errors) = parse(&tokens);
    assert!(errors.is_empty());
    let errors = analyze(&program);
    assert!(errors.is_empty());
}

#[test]
fn optional() {
    let src = "
def Optional = Number | null
let opt_none: Optional = null
let opt_some: Optional = 1
";
    let tokens = lex(src);
    let (program, errors) = parse(&tokens);
    assert!(errors.is_empty());
    let errors = super::analyze(&program);
    assert!(errors.is_empty());
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
    let tokens = lex(src);
    let (program, errors) = parse(&tokens);
    assert!(errors.is_empty());
    let errors = super::analyze(&program);
    assert!(errors.is_empty());
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
    let tokens = lex(src);
    let (program, errors) = parse(&tokens);
    assert!(errors.is_empty());
    let errors = super::analyze(&program);
    assert!(errors.is_empty());
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
    let tokens = lex(src);
    let (program, errors) = parse(&tokens);
    assert!(errors.is_empty());
    let errors = super::analyze(&program);
    assert!(errors.is_empty());
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

// #[test] fn self_reference() {
//     let src = "
// def Optional = Optional {}
// ";
//     let tokens = lex(src);
//     let (program, errors) = parse(&tokens);
//     assert!(errors.is_empty());
//     let errors = super::analyze(&program);
//     assert!(errors.is_empty());
// }
