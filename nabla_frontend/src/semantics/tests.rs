use crate::{
    ast::Global,
    eval::{eval, Value},
    lexer::{lex, LexerResult},
    parser::{parse, ParserResult},
    semantics::{
        self,
        error::{Error, ErrorMessage},
        uses,
        values::{self, ValuesResult},
        SemanticsResult,
    },
    GlobalIdent, ModuleAst,
};
use pretty_assertions::assert_eq;
use std::collections::HashMap;

#[test]
fn empty() {
    let src = "";
    let LexerResult { tokens, errors } = lex(src);
    assert_empty!(errors);
    let ParserResult { ast, errors } = parse(&tokens);
    assert_empty!(errors);
    let module_ast = ModuleAst::new(GlobalIdent::default(), ast);
    let SemanticsResult { errors, .. } = semantics::analyze(&module_ast);
    assert_empty!(errors);
}

#[test]
fn no_duplicate_use() {
    let src = "
use a
use b::{c d::e}
use f::g as h
";
    let LexerResult { tokens, errors } = lex(src);
    assert_empty!(errors);
    let ParserResult { ast, errors } = parse(&tokens);
    assert_empty!(errors);
    let module_ast = ModuleAst::new(GlobalIdent::default(), ast);
    let (_, errors) = uses::analyze(&module_ast);
    assert_empty!(errors);
}

#[test]
fn duplicate_use() {
    let src = "
use a::b
use c::b
";
    let LexerResult { tokens, errors } = lex(src);
    assert_empty!(errors);
    let ParserResult { ast, errors } = parse(&tokens);
    assert_empty!(errors);
    let module_ast = ModuleAst::new(GlobalIdent::default(), ast);
    let (_, errors) = uses::analyze(&module_ast);
    assert_eq!(
        vec![Error::new(
            ErrorMessage::DuplicateUse("b".to_string()),
            7..12
        )],
        errors
    );
}

#[test]
fn no_duplicate_use_alias() {
    let src = "
use a::b
use c::b as d
";
    let LexerResult { tokens, errors } = lex(src);
    assert_empty!(errors);
    let ParserResult { ast, errors } = parse(&tokens);
    assert_empty!(errors);
    let module_ast = ModuleAst::new(GlobalIdent::default(), ast);
    let (_, errors) = uses::analyze(&module_ast);
    assert_empty!(errors);
}

#[test]
fn empty_list() {
    let src = "
def EmptyList = []
EmptyList []
";
    let LexerResult { tokens, errors } = lex(src);
    assert_empty!(errors);
    let ParserResult { ast, errors } = parse(&tokens);
    assert_empty!(errors);
    let module_ast = ModuleAst::new(GlobalIdent::default(), ast);
    let SemanticsResult { errors, .. } = semantics::analyze(&module_ast);
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
    name = "Test"
    age = 0
}
"#;
    let LexerResult { tokens, errors } = lex(src);
    assert_empty!(errors);
    let ParserResult { ast, errors } = parse(&tokens);
    assert_empty!(errors);
    let module_ast = ModuleAst::new(GlobalIdent::default(), ast);
    let SemanticsResult { errors, .. } = semantics::analyze(&module_ast);
    assert_empty!(errors);
}

#[test]
fn optional() {
    let src = "
def Optional = Number | null
let opt_none: Optional = null
let opt_some: Optional = 1
";
    let LexerResult { tokens, errors } = lex(src);
    assert_empty!(errors);
    let ParserResult { ast, errors } = parse(&tokens);
    assert_empty!(errors);
    let module_ast = ModuleAst::new(GlobalIdent::default(), ast);
    let SemanticsResult { errors, .. } = semantics::analyze(&module_ast);
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
    let LexerResult { tokens, errors } = lex(src);
    assert_empty!(errors);
    let ParserResult { ast, errors } = parse(&tokens);
    assert_empty!(errors);
    let module_ast = ModuleAst::new(GlobalIdent::default(), ast);
    let SemanticsResult { errors, .. } = semantics::analyze(&module_ast);
    assert_empty!(errors);
    let init = module_ast
        .ast
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
    let LexerResult { tokens, errors } = lex(src);
    assert_empty!(errors);
    let ParserResult { ast, errors } = parse(&tokens);
    assert_empty!(errors);
    let module_ast = ModuleAst::new(GlobalIdent::default(), ast);
    let SemanticsResult { errors, .. } = semantics::analyze(&module_ast);
    assert_empty!(errors);
    let init = module_ast
        .ast
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
    let LexerResult { tokens, errors } = lex(src);
    assert_empty!(errors);
    let ParserResult { ast, errors } = parse(&tokens);
    assert_empty!(errors);
    let module_ast = ModuleAst::new(GlobalIdent::default(), ast);
    let SemanticsResult { errors, .. } = semantics::analyze(&module_ast);
    assert_empty!(errors);
    let init = module_ast
        .ast
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
    let LexerResult { tokens, errors } = lex(src);
    assert_empty!(errors);
    let ParserResult { ast, errors } = parse(&tokens);
    assert_empty!(errors);
    let module_ast = ModuleAst::new(GlobalIdent::default(), ast);
    let SemanticsResult { errors, .. } = semantics::analyze(&module_ast);
    assert_empty!(errors);
}

#[test]
fn self_reference_expr() {
    let src = "
def Type = Type {}
";
    let LexerResult { tokens, errors } = lex(src);
    assert_empty!(errors);
    let ParserResult { ast, errors } = parse(&tokens);
    assert_empty!(errors);
    let module_ast = ModuleAst::new(GlobalIdent::default(), ast);
    let SemanticsResult { errors, .. } = semantics::analyze(&module_ast);
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
    let LexerResult { tokens, errors } = lex(src);
    assert_empty!(errors);
    let ParserResult { ast, errors } = parse(&tokens);
    assert_empty!(errors);
    let module_ast = ModuleAst::new(GlobalIdent::default(), ast);
    let SemanticsResult { errors, .. } = semantics::analyze(&module_ast);
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
    let LexerResult { tokens, errors } = lex(src);
    assert_empty!(errors);
    let ParserResult { ast, errors } = parse(&tokens);
    assert_empty!(errors);
    let module_ast = ModuleAst::new(GlobalIdent::default(), ast);
    let SemanticsResult { errors, .. } = semantics::analyze(&module_ast);
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
    let LexerResult { tokens, errors } = lex(src);
    assert_empty!(errors);
    let ParserResult { ast, errors } = parse(&tokens);
    assert_empty!(errors);
    let module_ast = ModuleAst::new(GlobalIdent::default(), ast);
    let SemanticsResult { errors, .. } = semantics::analyze(&module_ast);
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
    let LexerResult { tokens, errors } = lex(src);
    assert_empty!(errors);
    let ParserResult { ast, errors } = parse(&tokens);
    assert_empty!(errors);
    let module_ast = ModuleAst::new(GlobalIdent::default(), ast);
    let SemanticsResult { errors, .. } = semantics::analyze(&module_ast);
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
    let LexerResult { tokens, errors } = lex(src);
    assert_empty!(errors);
    let ParserResult { ast, errors } = parse(&tokens);
    assert_empty!(errors);
    let module_ast = ModuleAst::new(GlobalIdent::default(), ast);
    let SemanticsResult { errors, .. } = semantics::analyze(&module_ast);
    assert_eq!(vec![Error::new(ErrorMessage::TypeMismatch, 31..32)], errors);
}

#[test]
fn union_in_let() {
    let src = r#"
let a: String = "A" | "a"
let b = "B" | "b"
"#;
    let LexerResult { tokens, errors } = lex(src);
    assert_empty!(errors);
    let ParserResult { ast, errors } = parse(&tokens);
    assert_empty!(errors);
    let module_ast = ModuleAst::new(GlobalIdent::default(), ast);
    let SemanticsResult { errors, .. } = semantics::analyze(&module_ast);
    assert_eq!(
        vec![
            Error::new(ErrorMessage::UninitializedLet, 1..15),
            Error::new(ErrorMessage::UninitializedLet, 16..27),
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
    let LexerResult { tokens, errors } = lex(src);
    assert_empty!(errors);
    let ParserResult { ast, errors } = parse(&tokens);
    assert_empty!(errors);
    let module_ast = ModuleAst::new(GlobalIdent::default(), ast);
    let SemanticsResult { errors, .. } = semantics::analyze(&module_ast);
    assert_eq!(
        vec![
            Error::new(ErrorMessage::UninitializedDefault, 9..21),
            Error::new(ErrorMessage::UninitializedDefault, 26..31),
        ],
        errors
    );
}

#[test]
fn assign_let() {
    let src = r#"
let a = "x"
{
    a = a
}
    "#;
    let LexerResult { tokens, errors } = lex(src);
    assert_empty!(errors);
    let ParserResult { ast, errors } = parse(&tokens);
    assert_empty!(errors);
    let module_ast = ModuleAst::new(GlobalIdent::default(), ast);
    let SemanticsResult {
        inits,
        symbol_table,
        errors,
    } = semantics::analyze(&module_ast);
    assert_empty!(errors);
    assert_eq!(
        HashMap::from([(
            GlobalIdent::default().extend("a".to_string()),
            Value::from("x")
        )]),
        symbol_table
    );
    assert_eq!(vec![Value::from([("a", "x")])], inits);
}

#[test]
fn default_init() {
    let src = r#"
def Config = {
    x = 0
}
Config {}
    "#;
    let LexerResult { tokens, errors } = lex(src);
    assert_empty!(errors);
    let ParserResult { ast, errors } = parse(&tokens);
    assert_empty!(errors);
    let module_ast = ModuleAst::new(GlobalIdent::default(), ast);
    let SemanticsResult {
        inits,
        symbol_table,
        errors,
    } = semantics::analyze(&module_ast);
    assert_empty!(errors);
    assert_eq!(
        HashMap::from([(
            GlobalIdent::default().extend("Config".to_string()),
            Value::from([("x", 0)]),
        )]),
        symbol_table
    );
    assert_eq!(vec![Value::from([("x", 0)])], inits);
}

#[test]
fn default_overwrite() {
    let src = r#"
def Config = {
    x: Number = 0
}
Config {
    x = 1
}
    "#;
    let LexerResult { tokens, errors } = lex(src);
    assert_empty!(errors);
    let ParserResult { ast, errors } = parse(&tokens);
    assert_empty!(errors);
    let module_ast = ModuleAst::new(GlobalIdent::default(), ast);
    let SemanticsResult {
        inits,
        symbol_table,
        errors,
    } = semantics::analyze(&module_ast);
    assert_empty!(errors);
    assert_eq!(
        HashMap::from([(
            GlobalIdent::default().extend("Config".to_string()),
            Value::from([("x", 0)]),
        )]),
        symbol_table
    );
    assert_eq!(vec![Value::from([("x", 1)])], inits);
}

#[test]
fn nested_default() {
    let src = r#"
def Config = {
    x: {
        y: Number = 0
        z: Number
    }
}
Config {
    x = {
        z = 1
    }
}
    "#;
    let LexerResult { tokens, errors } = lex(src);
    assert_empty!(errors);
    let ParserResult { ast, errors } = parse(&tokens);
    assert_empty!(errors);
    let module_ast = ModuleAst::new(GlobalIdent::default(), ast);
    let SemanticsResult {
        inits,
        symbol_table,
        errors,
    } = semantics::analyze(&module_ast);
    assert_empty!(errors);
    assert_eq!(
        HashMap::from([(
            GlobalIdent::default().extend("Config".to_string()),
            Value::from([(
                "x",
                Value::from([("y", Value::from(0)), ("z", Value::Unknown)])
            )]),
        )]),
        symbol_table
    );
    assert_eq!(vec![Value::from([("x", [("y", 0), ("z", 1)])])], inits);
}

#[test]
fn let_default() {
    let src = r#"
def Config = {
    x: Number = pi
}
let pi = 3.14
    "#;
    let LexerResult { tokens, errors } = lex(src);
    assert_empty!(errors);
    let ParserResult { ast, errors } = parse(&tokens);
    assert_empty!(errors);
    let module_ast = ModuleAst::new(GlobalIdent::default(), ast);
    let ValuesResult {
        symbol_table,
        errors,
        ..
    } = values::analyze(&module_ast);
    assert_empty!(errors);
    assert_eq!(
        HashMap::from([
            (
                GlobalIdent::default().extend("pi".to_string()),
                Value::from(3.14),
            ),
            (
                GlobalIdent::default().extend("Config".to_string()),
                Value::from([("x", 3.14)]),
            )
        ]),
        symbol_table
    );
}

#[test]
fn uninitialized_default() {
    let src = r#"
def Config = {
    x = {
        y: Number
    }
}
    "#;
    let LexerResult { tokens, errors } = lex(src);
    assert_empty!(errors);
    let ParserResult { ast, errors } = parse(&tokens);
    assert_empty!(errors);
    let module_ast = ModuleAst::new(GlobalIdent::default(), ast);
    let SemanticsResult { errors, .. } = semantics::analyze(&module_ast);
    assert_eq!(
        vec![Error::new(ErrorMessage::UninitializedDefault, 13..21)],
        errors
    );
}

#[test]
fn recursive_def_let() {
    let src = r#"
def Rec = {
    rec = rec
}
let rec = Rec {}
    "#;
    let LexerResult { tokens, errors } = lex(src);
    assert_empty!(errors);
    let ParserResult { ast, errors } = parse(&tokens);
    let module_ast = ModuleAst::new(GlobalIdent::default(), ast);
    assert_empty!(errors);
    let SemanticsResult { errors, .. } = semantics::analyze(&module_ast);
    assert_eq!(
        vec![
            Error::new(ErrorMessage::UninitializedDefault, 13..14),
            Error::new(ErrorMessage::UninitializedLet, 17..28),
        ],
        errors
    );
}

#[test]
fn recursive_let_let() {
    let src = r#"
let rec = {
     r = rec
}
    "#;
    let LexerResult { tokens, errors } = lex(src);
    assert_empty!(errors);
    let ParserResult { ast, errors } = parse(&tokens);
    assert_empty!(errors);
    let module_ast = ModuleAst::new(GlobalIdent::default(), ast);
    let SemanticsResult { errors, .. } = semantics::analyze(&module_ast);
    assert_eq!(
        vec![
            Error::new(ErrorMessage::UninitializedDefault, 13..14),
            Error::new(ErrorMessage::UninitializedLet, 1..17),
        ],
        errors
    );
}

// #[test]
// fn recursive_value_type() {
//     let src = r#"
// let x = {
//      x: x = x
// }
//     "#;
//     let LexerResult {tokens, errors} = lex(src);
//     assert_empty!(errors);
//     let ParserResult {ast, errors} = parse(&tokens);
//     assert_empty!(errors);
//     let module_ast = ModuleAst::new(GlobalIdent::default(), ast);
//     let TypeInfo { errors, .. } = types::analyze(&module_ast);
//     assert_empty!(errors);
//     let SemanticsResult { errors, ..} = values::analyze(&module_ast, &HashMap::new(), &HashMap::new());
//     assert_eq!(
//         vec![Error::new(ErrorMessage::RecursiveInit, 13..14)],
//         errors
//     );
// }

#[test]
fn immutable_let() {
    let src = r#"
let x = {
    a = "a"
}

x {
    a = "b"
}
    "#;
    let LexerResult { tokens, errors } = lex(src);
    assert_empty!(errors);
    let ParserResult { ast, errors } = parse(&tokens);
    assert_empty!(errors);
    let module_ast = ModuleAst::new(GlobalIdent::default(), ast);
    let SemanticsResult { errors, .. } = semantics::analyze(&module_ast);
    assert_eq!(
        vec![Error::new(
            ErrorMessage::ImmutableLet("x".to_string()),
            17..18
        ),],
        errors
    );
}
