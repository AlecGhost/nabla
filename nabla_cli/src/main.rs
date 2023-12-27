use clap::Parser;
use nabla_backend::to_json_value;
use nabla_frontend::{
    lexer::lex,
    parser::parse,
    semantics::{analyze, TypeInfo, values},
    token::TextRange,
};
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
    for error in errors {
        let range = convert_text_range(&src, &error.range);
        println!(
            "Line {}, Char {}: {}",
            range.start.line, range.start.char, error
        );
    }
    let (program, errors) = parse(&tokens);
    if !errors.is_empty() {
        valid = false
    }
    for error in errors {
        let text_range = tokens[error.range.start].range.start..tokens[error.range.end].range.end;
        let range = convert_text_range(&src, &text_range);
        println!(
            "Line {}, Char {}: {}",
            range.start.line, range.start.char, error
        );
    }
    let TypeInfo { errors, .. } = analyze(&program);
    if !errors.is_empty() {
        valid = false
    }
    for error in errors {
        let text_range = tokens[error.range.start].range.start..tokens[error.range.end].range.end;
        let range = convert_text_range(&src, &text_range);
        println!(
            "Line {}, Char {}: {}",
            range.start.line, range.start.char, error
        );
    }
    let (inits, _, errors) = values::analyze(&program);
    if !errors.is_empty() {
        valid = false
    }
    for error in errors {
        let text_range = tokens[error.range.start].range.start..tokens[error.range.end].range.end;
        let range = convert_text_range(&src, &text_range);
        println!(
            "Line {}, Char {}: {}",
            range.start.line, range.start.char, error
        );
    }
    if valid {
        if let Some(init) = inits.first() {
            if let Some(json) = to_json_value(init.clone()) {
                let pretty_json = serde_json::to_string_pretty(&json).expect("Converting value to string failed");
                println!("{}", pretty_json);
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Pos {
    line: usize,
    char: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PosRange {
    start: Pos,
    _end: Pos,
}

fn convert_text_range(text: &str, range: &TextRange) -> PosRange {
    let before_range = &text[..range.start];
    let start = before_range
        .split('\n')
        .enumerate()
        .last()
        .map(|(line_number, last_line)| Pos {
            line: line_number,
            char: last_line.len(),
        })
        .expect("Split must yield at least one element");
    let in_range = &text[range.clone()];
    let end = in_range
        .split('\n')
        .enumerate()
        .last()
        .map(|(line_number, last_line)| Pos {
            line: line_number + start.line,
            char: if line_number == 0 {
                // end is on the same line as start, therefore the char positions must be added
                start.char + last_line.len()
            } else {
                last_line.len()
            },
        })
        .expect("Split must yield at least one element");
    PosRange { start, _end: end }
}
