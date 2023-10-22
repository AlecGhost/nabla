use clap::Parser;
use nabla_backend::to_json_string;
use nabla_frontend::{ast::Global, eval::eval, lexer::lex, parser::parse, semantics::analyze};
use std::path::PathBuf;

#[derive(Debug, Parser)]
struct Args {
    file: PathBuf,
}

fn main() {
    let args = Args::parse();
    let src = std::fs::read_to_string(args.file).expect("Could not open file");
    let mut valid = true;
    let (tokens, errors) = lex(&src);
    if !errors.is_empty() {
        valid = false
    }
    print_errors(errors);
    let (program, errors) = parse(&tokens);
    if !errors.is_empty() {
        valid = false
    }
    print_errors(errors);
    let errors = analyze(&program);
    if !errors.is_empty() {
        valid = false
    }
    print_errors(errors);
    if valid {
        if let Some(init) = program.globals.iter().find_map(|global| match global {
            Global::Init(init) => Some(init),
            _ => None,
        }) {
            let value = eval(init);
            if let Some(json) = to_json_string(value) {
                println!("{json}");
            }
        }
    }
}

fn print_errors<E: std::fmt::Display>(errors: Vec<E>) {
    for error in errors {
        println!("{error}");
    }
}
