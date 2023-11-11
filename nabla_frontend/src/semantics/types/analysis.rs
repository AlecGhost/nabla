use crate::{
    ast::*,
    semantics::{
        error::{Error, ErrorMessage},
        types::{Rule, TypeDescription, TypeInfo},
    },
    GlobalIdent,
};

use super::RuleIndex;

pub(super) fn analyze_def<'a>(def: &'a Def, type_info: &mut TypeInfo<'a>) {
    analyze_binding(
        def.name.as_ref(),
        def.type_expr.as_ref(),
        def.expr.as_ref(),
        type_info,
    );
}

pub(super) fn analyze_let<'a>(l: &'a Let, type_info: &mut TypeInfo<'a>) {
    analyze_binding(
        l.name.as_ref(),
        l.type_expr.as_ref(),
        l.expr.as_ref(),
        type_info,
    );
}

fn analyze_binding<'a>(
    name: Option<&'a Ident>,
    type_expr: Option<&'a Expr>,
    expr: Option<&'a Expr>,
    type_info: &mut TypeInfo<'a>,
) {
    fn check_self_reference(ident: &Ident, expr: Option<&Expr>) -> bool {
        matches!(expr, Some(Expr::Single(Single::Named(Named { name, .. }))) if ident == name)
    }

    let TypeInfo {
        ref mut rules,
        ref mut assertions,
        ref mut idents,
        ref mut errors,
    } = type_info;
    if let Some(name) = name {
        if check_self_reference(name, type_expr) || check_self_reference(name, expr) {
            errors.push(Error::new(
                ErrorMessage::SelfReference(name.name.clone()),
                name.info.range.clone(),
            ));
            return;
        }
    }
    let ident_entry = name.as_ref().and_then(|name| {
        use std::collections::hash_map::Entry;
        match idents.entry(name) {
            Entry::Vacant(entry) => Some(entry),
            Entry::Occupied(_) => {
                errors.push(Error::new(
                    ErrorMessage::Redeclaration(name.name.clone()),
                    name.info.range.clone(),
                ));
                None
            }
        }
    });
    let rule_index = match (
        type_expr
            .as_ref()
            .map(|type_expr| type_expr.analyze(rules, assertions)),
        expr.as_ref().map(|expr| expr.analyze(rules, assertions)),
    ) {
        (Some(type_expr_index), Some(expr_index)) => {
            assertions.push((type_expr_index, expr_index));
            Some(type_expr_index)
        }
        (Some(index), None) | (None, Some(index)) => Some(index),
        (None, None) => None,
    };
    if let (Some(entry), Some(index)) = (ident_entry, rule_index) {
        entry.insert(index);
    }
}

pub(super) fn analyze_use<'a>(u: &'a Use, type_info: &mut TypeInfo<'a>) {
    /// Analyzes the body and returns whether the `UseKind` was `Single`.
    fn analyze_body<'a>(
        body: &'a UseBody,
        type_info: &mut TypeInfo<'a>,
        path_stack: &mut Vec<String>,
    ) -> bool {
        if let Some(kind) = &body.kind {
            match kind {
                UseKind::All(info) => {
                    type_info.errors.push(Error {
                        message: ErrorMessage::Unsupported("glob import".to_string()),
                        range: info.range.clone(),
                    });
                    false
                }
                UseKind::Single(item) => {
                    analyze_item(item, type_info, path_stack);
                    true
                }
                UseKind::Multiple(items) => {
                    items.items.iter().for_each(|item| {
                        analyze_item(item, type_info, path_stack);
                    });
                    false
                }
                UseKind::Error(_) => false,
            }
        } else {
            false
        }
    }

    fn analyze_item<'a>(
        item: &'a UseItem,
        type_info: &mut TypeInfo<'a>,
        path_stack: &mut Vec<String>,
    ) {
        if let Some(body) = &item.body {
            path_stack.push(item.name.name.clone());
            let is_single = analyze_body(body, type_info, path_stack);
            if !is_single {
                if let Some(alias) = &item.alias {
                    type_info.errors.push(Error::new(
                        ErrorMessage::AliasingNonSingle,
                        alias.info.range.clone(),
                    ));
                }
            }
            path_stack.pop();
        } else {
            let TypeInfo {
                ref mut rules,
                ref mut idents,
                ref mut errors,
                ..
            } = type_info;
            path_stack.push(item.name.name.clone());
            let import_rule = Rule {
                type_description: TypeDescription::Import(GlobalIdent {
                    root: path_stack
                        .first()
                        .expect("Path stack must have a root element")
                        .clone(),
                    path: path_stack[1..].to_vec(),
                }),
                info: item.info.clone(),
            };
            rules.push(import_rule);
            path_stack.pop();
            let import_rule_index = rule_index(rules);
            let name = item
                .alias
                .as_ref()
                .and_then(|alias| alias.name.as_ref())
                .and_then(|alias_name| match alias_name {
                    AliasName::Ident(ident) => Some(ident),
                    AliasName::String(_) => None,
                })
                .unwrap_or(&item.name);
            use std::collections::hash_map::Entry;
            match idents.entry(name) {
                Entry::Vacant(entry) => {
                    entry.insert(import_rule_index);
                }
                Entry::Occupied(_) => {
                    errors.push(Error::new(
                        ErrorMessage::Redeclaration(name.name.clone()),
                        name.info.range.clone(),
                    ));
                }
            }
        }
    }

    if let (Some(root), Some(body)) = (&u.name, &u.body) {
        let mut path_stack = vec![root.name.clone()];
        if !analyze_body(body, type_info, &mut path_stack) {
            let import_rule = Rule {
                type_description: TypeDescription::Import(GlobalIdent {
                    root: root.name.clone(),
                    path: Vec::new(),
                }),
                info: root.info.clone(),
            };
            type_info.rules.push(import_rule);
        }
    }
}

#[inline]
const fn rule_index(rules: &[Rule]) -> RuleIndex {
    rules.len() - 1
}

pub(super) trait TypeAnalyzer {
    fn analyze(
        &self,
        rules: &mut Vec<Rule>,
        assertions: &mut Vec<(RuleIndex, RuleIndex)>,
    ) -> RuleIndex;
}

impl TypeAnalyzer for Expr {
    fn analyze(
        &self,
        rules: &mut Vec<Rule>,
        assertions: &mut Vec<(RuleIndex, RuleIndex)>,
    ) -> RuleIndex {
        match self {
            Self::Union(union) => union.analyze(rules, assertions),
            Self::Single(single) => single.analyze(rules, assertions),
            Self::Error(info) => {
                rules.push(Rule {
                    type_description: TypeDescription::Unknown,
                    info: info.clone(),
                });
                rule_index(rules)
            }
        }
    }
}

impl TypeAnalyzer for Union {
    fn analyze(
        &self,
        rules: &mut Vec<Rule>,
        assertions: &mut Vec<(RuleIndex, RuleIndex)>,
    ) -> RuleIndex {
        let mut inner_rule_indices = Vec::with_capacity(self.alternatives.len() + 1);
        inner_rule_indices.push(self.single.analyze(rules, assertions));
        self.alternatives
            .iter()
            .flat_map(|alternative| {
                alternative
                    .single
                    .as_ref()
                    .map(|single| single.analyze(rules, assertions))
            })
            .for_each(|rule| inner_rule_indices.push(rule));
        rules.push(Rule {
            type_description: TypeDescription::Union(inner_rule_indices),
            info: self.info.clone(),
        });
        rule_index(rules)
    }
}

impl TypeAnalyzer for Single {
    fn analyze(
        &self,
        rules: &mut Vec<Rule>,
        assertions: &mut Vec<(RuleIndex, RuleIndex)>,
    ) -> RuleIndex {
        match self {
            Self::Struct(s) => s.analyze(rules, assertions),
            Self::List(list) => list.analyze(rules, assertions),
            Self::Named(named) => named.analyze(rules, assertions),
            Self::Primitive(primitive) => primitive.analyze(rules, assertions),
        }
    }
}

impl TypeAnalyzer for StructOrList {
    fn analyze(
        &self,
        rules: &mut Vec<Rule>,
        assertions: &mut Vec<(RuleIndex, RuleIndex)>,
    ) -> RuleIndex {
        match self {
            Self::Struct(s) => s.analyze(rules, assertions),
            Self::List(l) => l.analyze(rules, assertions),
        }
    }
}

impl TypeAnalyzer for Struct {
    fn analyze(
        &self,
        rules: &mut Vec<Rule>,
        assertions: &mut Vec<(RuleIndex, RuleIndex)>,
    ) -> RuleIndex {
        let field_rule_indices = self
            .fields
            .iter()
            .map(|field| (field.name.name.clone(), field.analyze(rules, assertions)))
            .collect();
        rules.push(Rule {
            type_description: TypeDescription::Struct(field_rule_indices),
            info: self.info.clone(),
        });
        rule_index(rules)
    }
}

impl TypeAnalyzer for StructField {
    fn analyze(
        &self,
        rules: &mut Vec<Rule>,
        assertions: &mut Vec<(RuleIndex, RuleIndex)>,
    ) -> RuleIndex {
        let info = self.info.clone();
        let rule = match (
            self.type_expr
                .as_ref()
                .map(|type_expr| type_expr.analyze(rules, assertions)),
            self.expr
                .as_ref()
                .map(|expr| expr.analyze(rules, assertions)),
        ) {
            (Some(type_expr_index), Some(expr_index)) => {
                assertions.push((type_expr_index, expr_index));
                Rule {
                    type_description: TypeDescription::Rule(type_expr_index),
                    info,
                }
            }
            (Some(expr_index), None) | (None, Some(expr_index)) => Rule {
                type_description: TypeDescription::Rule(expr_index),
                info,
            },
            (None, None) => Rule {
                type_description: TypeDescription::Unknown,
                info,
            },
        };
        rules.push(rule);
        rule_index(rules)
    }
}

impl TypeAnalyzer for List {
    fn analyze(
        &self,
        rules: &mut Vec<Rule>,
        assertions: &mut Vec<(RuleIndex, RuleIndex)>,
    ) -> RuleIndex {
        let inner_rule_indices = self
            .exprs
            .iter()
            .map(|expr| expr.analyze(rules, assertions))
            .collect();
        rules.push(Rule {
            type_description: TypeDescription::List(inner_rule_indices),
            info: self.info.clone(),
        });
        rule_index(rules)
    }
}

impl TypeAnalyzer for Named {
    fn analyze(
        &self,
        rules: &mut Vec<Rule>,
        assertions: &mut Vec<(RuleIndex, RuleIndex)>,
    ) -> RuleIndex {
        let named_rule = Rule {
            type_description: TypeDescription::Ident(self.name.clone()),
            info: self.name.info.clone(),
        };
        rules.push(named_rule);
        let named_rule_index = rule_index(rules);
        if let Some(expr_rule_index) = self
            .expr
            .as_ref()
            .map(|expr| expr.analyze(rules, assertions))
        {
            assertions.push((named_rule_index, expr_rule_index));
        };
        named_rule_index
    }
}

impl TypeAnalyzer for Primitive {
    fn analyze(
        &self,
        rules: &mut Vec<Rule>,
        _assertions: &mut Vec<(RuleIndex, RuleIndex)>,
    ) -> RuleIndex {
        rules.push(Rule {
            type_description: TypeDescription::Primitive(self.clone()),
            info: AstInfo::new(0..0), // fake info
        });
        rule_index(rules)
    }
}
