use crate::{
    ast::{Global, Use, UseBody, UseItem, UseKind},
    semantics::error::{Error, ErrorMessage},
    token::ToTokenRange,
    GlobalIdent, ModuleAst,
};
use std::collections::HashMap;

pub fn analyze(module_ast: &ModuleAst) -> (HashMap<String, GlobalIdent>, Vec<Error>) {
    module_ast
        .ast
        .globals
        .iter()
        .filter_map(|global| match global {
            Global::Use(u) => Some(u),
            _ => None,
        })
        .map(|u| {
            let (idents, errors) = analyze_use(u);
            (u, idents, errors)
        })
        .fold(
            (HashMap::new(), Vec::new()),
            |(mut idents, mut errors), (u, new_idents, new_errors)| {
                use std::collections::hash_map::Entry;
                for (key, value) in new_idents {
                    match idents.entry(key) {
                        Entry::Occupied(entry) => {
                            errors.push(Error::new(
                                ErrorMessage::DuplicateUse(entry.key().clone()),
                                u.info.to_token_range(),
                            ));
                        }
                        Entry::Vacant(entry) => {
                            entry.insert(value);
                        }
                    }
                }
                errors.extend(new_errors);
                (idents, errors)
            },
        )
}

fn analyze_use(u: &Use) -> (HashMap<String, GlobalIdent>, Vec<Error>) {
    match (&u.name, &u.body) {
        (Some(root), Some(body)) => {
            let mut path_stack = vec![root.name.clone()];
            let (idents, errors, _) = analyze_body(body, &mut path_stack);
            (idents, errors)
        }
        (Some(root), None) => {
            let ident = GlobalIdent {
                root: root.name.clone(),
                path: Vec::new(),
            };
            (
                HashMap::from([(u.identifier().expect("name is present").name.clone(), ident)]),
                Vec::new(),
            )
        }
        _ => (HashMap::new(), Vec::new()),
    }
}
/// Analyzes the body and returns whether the `UseKind` was `Single`.
fn analyze_body(
    body: &UseBody,
    path_stack: &mut Vec<String>,
) -> (HashMap<String, GlobalIdent>, Vec<Error>, bool) {
    body.kind.as_ref().map_or_else(
        || (HashMap::new(), Vec::new(), false),
        |kind| match kind {
            UseKind::All(info) => (
                HashMap::new(),
                vec![
                    (Error {
                        message: ErrorMessage::Unsupported("glob import".to_string()),
                        range: info.to_token_range(),
                    }),
                ],
                false,
            ),
            UseKind::Single(item) => {
                let (idents, errors) = analyze_item(item, path_stack);
                (idents, errors, true)
            }
            UseKind::Multiple(items) => {
                let (idents, errors) = items
                    .items
                    .iter()
                    .flatten()
                    .map(|item| {
                        let (idents, errors) = analyze_item(item, path_stack);
                        (item, idents, errors)
                    })
                    .fold(
                        (HashMap::new(), Vec::new()),
                        |(mut idents, mut errors), (item, new_idents, new_errors)| {
                            use std::collections::hash_map::Entry;
                            for (key, value) in new_idents {
                                match idents.entry(key) {
                                    Entry::Occupied(entry) => {
                                        errors.push(Error::new(
                                            ErrorMessage::DuplicateUse(entry.key().clone()),
                                            item.info.to_token_range(),
                                        ));
                                    }
                                    Entry::Vacant(entry) => {
                                        entry.insert(value);
                                    }
                                }
                            }
                            errors.extend(new_errors);
                            (idents, errors)
                        },
                    );
                (idents, errors, false)
            }
            UseKind::Error(_) => (HashMap::new(), Vec::new(), false),
        },
    )
}

fn analyze_item(
    item: &UseItem,
    path_stack: &mut Vec<String>,
) -> (HashMap<String, GlobalIdent>, Vec<Error>) {
    path_stack.push(item.name.name.clone());
    if let Some(body) = &item.body {
        let (idents, mut errors, is_single) = analyze_body(body, path_stack);
        if !is_single {
            if let Some(alias) = &item.alias {
                errors.push(Error::new(
                    ErrorMessage::AliasingNonSingle,
                    alias.info.to_token_range(),
                ));
            }
        }
        path_stack.pop();
        (idents, errors)
    } else {
        // is a terminal item
        let ident = GlobalIdent {
            root: path_stack
                .first()
                .expect("Path stack must have a root element")
                .clone(),
            path: path_stack[1..].to_vec(),
        };
        path_stack.pop();
        (
            HashMap::from([(item.identifier().name.clone(), ident)]),
            Vec::new(),
        )
    }
}
