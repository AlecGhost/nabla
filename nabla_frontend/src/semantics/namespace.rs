use crate::{
    ast::{Expr, Global, Named, Single, StructOrList, TypedExpr},
    semantics::{
        error::{Error, ErrorMessage},
        types, BindingMap, Errors, Namespace,
    },
    token::ToTokenRange,
    ModuleAst,
};

#[derive(Clone, Debug)]
pub enum Binding {
    Def,
    Let,
}

pub struct NamespaceResult {
    pub namespace: Namespace,
    pub bindings: BindingMap,
    pub errors: Errors,
}

pub fn analyze(uses: &Namespace, module_ast: &ModuleAst) -> NamespaceResult {
    let module_name = module_ast.name.clone();
    let mut namespace = uses.clone();
    let mut bindings = BindingMap::new();
    let mut errors = Errors::new();
    for (ident, global_ident, binding) in
        module_ast
            .ast
            .globals
            .iter()
            .flat_map(|global| match global {
                Global::Def(d) => d.name.as_ref().map(|ident| {
                    (
                        ident,
                        module_name.clone().extend(ident.name.clone()),
                        Binding::Def,
                    )
                }),
                Global::Let(l) => l.name.as_ref().map(|ident| {
                    (
                        ident,
                        module_name.clone().extend(ident.name.clone()),
                        Binding::Let,
                    )
                }),
                _ => None,
            })
    {
        use std::collections::hash_map::Entry;
        match namespace.entry(ident.name.clone()) {
            Entry::Vacant(entry) => {
                entry.insert(global_ident.clone());
                bindings.insert(global_ident, binding);
            }
            Entry::Occupied(_) => {
                errors.push(Error::new(
                    ErrorMessage::Redeclaration(ident.name.clone()),
                    ident.info.to_token_range(),
                ));
            }
        }
    }
    for named in module_ast
        .ast
        .globals
        .iter()
        .flat_map(|global| match global {
            Global::Def(def) => get_named_typed_expr(def),
            Global::Let(l) => get_named_typed_expr(l),
            Global::Init(init) => get_named(init),
            Global::Use(_) | Global::Error(_) => Vec::new(),
        })
    {
        match namespace.get(&named.name.name) {
            None => {
                if !(named.inner_names.is_empty()
                    && types::BuiltInType::into_iter()
                        .map(|built_in| built_in.as_str())
                        .any(|built_in| named.name.name == built_in))
                {
                    errors.push(Error::new(
                        ErrorMessage::UndefinedIdent(named.name.name.clone()),
                        named.name.info.to_token_range(),
                    ));
                }
            }
            Some(_global_ident) => { /* TODO: check if inner names actually exist */ }
        }
    }

    NamespaceResult {
        namespace,
        bindings,
        errors,
    }
}

fn get_named(expr: &Expr) -> Vec<&Named> {
    match expr {
        Expr::Union(union) => [
            vec![&union.single],
            union
                .alternatives
                .iter()
                .flat_map(|alt| &alt.single)
                .collect(),
        ]
        .concat()
        .into_iter()
        .flat_map(get_named_single)
        .collect(),
        Expr::Single(single) => get_named_single(single),
        Expr::Error(_) => Vec::new(),
    }
}

fn get_named_typed_expr(typed_expr: &impl TypedExpr) -> Vec<&Named> {
    [
        typed_expr.type_expr().map(get_named).unwrap_or_default(),
        typed_expr.expr().map(get_named).unwrap_or_default(),
    ]
    .concat()
}

fn get_named_single(single: &Single) -> Vec<&Named> {
    match single {
        Single::Named(named) => [
            vec![named],
            named
                .expr
                .as_ref()
                .map(|struct_or_list| match struct_or_list {
                    StructOrList::List(l) => l.exprs.iter().flat_map(get_named).collect(),
                    StructOrList::Struct(s) => s
                        .fields
                        .iter()
                        .flatten()
                        .flat_map(get_named_typed_expr)
                        .collect(),
                })
                .unwrap_or_default(),
        ]
        .concat(),
        Single::List(l) => l.exprs.iter().flat_map(get_named).collect(),
        Single::Struct(s) => s
            .fields
            .iter()
            .flatten()
            .flat_map(get_named_typed_expr)
            .collect(),
        Single::Primitive(_) => Vec::new(),
    }
}
