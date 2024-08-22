use crate::{
    ast::{AstInfo, Global, Use, UseBody, UseItem, UseKind},
    semantics::error::{Error, ErrorMessage},
    token::ToTokenRange,
    GlobalIdent, ModuleAst,
};

type UseTable = std::collections::HashMap<String, GlobalIdent>;
type Errors = Vec<Error>;

/// Analyze use statements for a module.
///
/// Returns (_use table_, _errors_).
///
/// The use table is a map from the module-local identifier as a String to the global identifier.
pub fn analyze(module_ast: &ModuleAst) -> (UseTable, Errors) {
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
            (&u.info, idents, errors)
        })
        .fold((UseTable::new(), Errors::new()), fold_uses)
}

/// Fold uses into a single map.
/// If a name is used twice, a duplicate error is reported.
fn fold_uses(
    (mut idents, mut errors): (UseTable, Errors),
    (info, new_idents, new_errors): (&AstInfo, UseTable, Errors),
) -> (UseTable, Errors) {
    use std::collections::hash_map::Entry;
    for (key, value) in new_idents {
        match idents.entry(key) {
            Entry::Occupied(entry) => {
                errors.push(Error::new(
                    ErrorMessage::DuplicateUse(entry.key().clone()),
                    info.to_token_range(),
                ));
            }
            Entry::Vacant(entry) => {
                entry.insert(value);
            }
        }
    }
    errors.extend(new_errors);
    (idents, errors)
}

fn analyze_use(u: &Use) -> (UseTable, Errors) {
    match (&u.name, &u.body) {
        (Some(root), Some(body)) => {
            // the path stack is used to keep track of the module hierarchy.
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
                UseTable::from([(u.identifier().expect("name is present").name.clone(), ident)]),
                Errors::new(),
            )
        }
        _ => (UseTable::new(), Errors::new()),
    }
}

/// Analyzes the body and returns whether the `UseKind` was `Single`.
fn analyze_body(body: &UseBody, path_stack: &mut Vec<String>) -> (UseTable, Errors, bool) {
    body.kind.as_ref().map_or_else(
        || (UseTable::new(), Errors::new(), false),
        |kind| match kind {
            UseKind::All(info) => (
                UseTable::new(),
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
                        (&item.info, idents, errors)
                    })
                    .fold((UseTable::new(), Errors::new()), fold_uses);
                (idents, errors, false)
            }
            UseKind::Error(_) => (UseTable::new(), Errors::new(), false),
        },
    )
}

fn analyze_item(item: &UseItem, path_stack: &mut Vec<String>) -> (UseTable, Errors) {
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
            UseTable::from([(item.identifier().name.clone(), ident)]),
            Errors::new(),
        )
    }
}
