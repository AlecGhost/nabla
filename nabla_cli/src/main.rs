use clap::Parser;
use nabla_backend::{to_json_value, to_toml_value, to_xml_value, to_yaml_value};
use nabla_frontend::{lexer, parser, semantics, token::TextRange};
use std::path::PathBuf;

macro_rules! printerr {
    ($errors:expr, $src:expr, $tokens:expr) => {
        for error in $errors {
            let text_range =
                $tokens[error.range.start].range.start..$tokens[error.range.end].range.end;
            let range = convert_text_range(&$src, &text_range);
            println!(
                "Line {}, Char {}: {}",
                range.start.line, range.start.char, error
            );
        }
    };
}

#[derive(Clone, Debug, Default, clap::ValueEnum)]
enum Target {
    #[default]
    Json,
    Yaml,
    Toml,
    Xml,
}

#[derive(Debug, Parser)]
struct Args {
    file: PathBuf,
    #[clap(short, long, default_value = "json")]
    target: Target,
}

fn main() -> color_eyre::Result<()> {
    let args = Args::parse();
    let src = std::fs::read_to_string(args.file).expect("Could not open file");
    let mut valid = true;
    let (tokens, errors) = lexer::lex(&src);
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
    let (ast, errors) = parser::parse(&tokens);
    if !errors.is_empty() {
        valid = false
    }
    printerr!(errors, src, tokens);
    let (inits, _, errors) = semantics::analyze(&ast);
    if !errors.is_empty() {
        valid = false
    }
    printerr!(errors, src, tokens);
    if valid {
        if let Some(init) = inits.first() {
            match args.target {
                Target::Json => {
                    let json = to_json_value(init.clone())?;
                    let pretty_json = serde_json::to_string_pretty(&json)
                        .expect("Converting value to json string failed");
                    println!("{}", pretty_json);
                }
                Target::Yaml => {
                    let yaml = to_yaml_value(init.clone())?;
                    let pretty_yaml = serde_yaml::to_string(&yaml)
                        .expect("Converting value to yaml string failed");
                    println!("{}", pretty_yaml);
                }
                Target::Toml => {
                    let toml = to_toml_value(init.clone())?;
                    let pretty_toml = toml::to_string_pretty(&toml)
                        .expect("Converting value to yaml string failed");
                    println!("{}", pretty_toml);
                }
                Target::Xml => {
                    let element = to_xml_value(init.clone(), "root")?;
                    let mut xml = xml_builder::XMLBuilder::new().build();
                    xml.set_root_element(element);
                    xml.generate(std::io::stdout())
                        .expect("Generation XML failed");
                }
            }
        } else {
            println!("No errors detected.");
        }
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Pos {
    line: usize,
    char: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PosRange {
    start: Pos,
    end: Pos,
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
    PosRange { start, end }
}
