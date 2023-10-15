use crate::{
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
    let program = parse(&tokens);
    let errors = analyze(&program);
    assert_eq!(Vec::<Error>::new(), errors);
}

#[test]
fn no_reimport() {
    let src = "
use a
use b::{c d::e}
use f::g as h
";
    let tokens = lex(src);
    let program = parse(&tokens);
    let errors = analyze(&program);
    assert_eq!(Vec::<Error>::new(), errors);
}

#[test]
fn reimport() {
    let src = "
use a::b
use c::b
";
    let tokens = lex(src);
    let program = parse(&tokens);
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
    let program = parse(&tokens);
    let errors = analyze(&program);
    assert_eq!(Vec::<Error>::new(), errors);
}

#[test] fn empty_list() {
    let src = "
def EmptyList = []
EmptyList [ 0 ]
";
    let tokens = lex(src);
    let program = parse(&tokens);
    let errors = analyze(&program);
    assert_eq!(Vec::<Error>::new(), errors);
}

#[test] fn struct_fields() {
    let src = "
def Person = {
    name: String
    age: Number
}
Person {
    name: \"Test\"
    age: true
}
";
    let tokens = lex(src);
    let program = parse(&tokens);
    let errors = analyze(&program);
    assert_eq!(Vec::<Error>::new(), errors);
}

#[test] fn optional() {
    let src = "
def Optional = Number | None
Optional 1
";
    let tokens = lex(src);
    let program = parse(&tokens);
    let errors = super::analyze(&program);
    assert_eq!(Vec::<Error>::new(), errors);
}

// #[test] fn self_reference() {
//     let src = "
// def Optional = Optional {}
// ";
//     let tokens = lex(src);
//     let program = parse(&tokens);
//     let errors = super::analyze(&program);
//     assert_eq!(Vec::<Error>::new(), errors);
// }
