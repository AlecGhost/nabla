use clap::Parser;
use nabla_backend::{to_json_value, to_yaml_value};
use nabla_frontend::{
    lexer::lex,
    parser::parse,
    semantics::{analyze, values},
    token::TextRange,
};
use std::path::PathBuf;

#[derive(Clone, Debug, Default, clap::ValueEnum)]
enum Target {
    #[default]
    Json,
    Yaml,
}

#[derive(Debug, Parser)]
struct Args {
    file: PathBuf,
    #[clap(short, long, default_value = "json")]
    target: Target,
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
    let errors = analyze(&program);
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
            match args.target {
                Target::Json => {
                    if let Some(json) = to_json_value(init.clone()) {
                        let pretty_json = serde_json::to_string_pretty(&json)
                            .expect("Converting value to json string failed");
                        println!("{}", pretty_json);
                    }
                }
                Target::Yaml => {
                    if let Some(yaml) = to_yaml_value(init.clone()) {
                        let pretty_yaml = serde_yaml::to_string(&yaml)
                            .expect("Converting value to yaml string failed");
                        println!("{}", pretty_yaml);
                    }
                }
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
