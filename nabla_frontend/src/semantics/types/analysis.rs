use std::ops::Deref;

use crate::{
    ast::*,
    semantics::{
        error::{Error, ErrorMessage},
        namespace::Binding,
        types::{BuiltInType, Context, Rule, RuleIndex, TypeDescription, TypesResult},
        BindingMap, Namespace,
    },
    token::ToTokenRange,
};

pub(super) fn analyze_def(
    def: &Def,
    types_result: &mut TypesResult,
    space_info: (&Namespace, &BindingMap),
) -> Option<RuleIndex> {
    analyze_binding(
        def.name.as_ref(),
        def.type_expr.as_ref(),
        def.expr.as_ref(),
        types_result,
        Context::TypeExpr,
        space_info,
    )
}

pub(super) fn analyze_let(
    l: &Let,
    types_result: &mut TypesResult,
    space_info: (&Namespace, &BindingMap),
) -> Option<RuleIndex> {
    analyze_binding(
        l.name.as_ref(),
        l.type_expr.as_ref(),
        l.expr.as_ref(),
        types_result,
        Context::Expr,
        space_info,
    )
}

fn analyze_binding<'a>(
    name: Option<&'a Ident>,
    type_expr: Option<&'a Expr>,
    expr: Option<&'a Expr>,
    types_result: &mut TypesResult,
    context: Context,
    space_info: (&Namespace, &BindingMap),
) -> Option<RuleIndex> {
    fn check_self_reference(ident: &Ident, expr: Option<&Expr>) -> bool {
        matches!(expr, Some(Expr::Single(Single::Named(Named { name, .. }))) if ident == name)
    }

    if let Some(name) = name {
        if check_self_reference(name, type_expr) || check_self_reference(name, expr) {
            types_result.errors.push(Error::new(
                ErrorMessage::SelfReference(name.name.clone()),
                name.info.to_token_range(),
            ));
            return None;
        }
    }
    match (
        type_expr
            .as_ref()
            .map(|type_expr| type_expr.analyze(types_result, Context::TypeExpr, space_info)),
        expr.as_ref()
            .map(|expr| expr.analyze(types_result, context, space_info)),
    ) {
        (Some(type_expr_index), Some(expr_index)) => {
            if !(matches!(context, Context::Expr) && is_union(types_result, expr_index)) {
                types_result.assertions.push((type_expr_index, expr_index));
            }
            Some(type_expr_index)
        }
        (Some(index), None) | (None, Some(index)) => Some(index),
        (None, None) => None,
    }
}

#[inline]
const fn rule_index(rules: &[Rule]) -> RuleIndex {
    rules.len() - 1
}

pub(super) trait TypeAnalyzer {
    fn analyze(
        &self,
        types_result: &mut TypesResult,
        context: Context,
        space_info: (&Namespace, &BindingMap),
    ) -> RuleIndex;
}

impl TypeAnalyzer for Expr {
    fn analyze(
        &self,
        types_result: &mut TypesResult,
        context: Context,
        space_info: (&Namespace, &BindingMap),
    ) -> RuleIndex {
        let rules = &mut types_result.rules;
        match self {
            Self::Union(union) => union.analyze(types_result, context, space_info),
            Self::Single(single) => single.analyze(types_result, context, space_info),
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
        types_result: &mut TypesResult,
        context: Context,
        space_info: (&Namespace, &BindingMap),
    ) -> RuleIndex {
        let mut inner_rule_indices = Vec::with_capacity(self.alternatives.len() + 1);
        inner_rule_indices.push(self.single.analyze(types_result, context, space_info));
        self.alternatives
            .iter()
            .flat_map(|alternative| {
                alternative
                    .single
                    .as_ref()
                    .map(|single| single.analyze(types_result, context, space_info))
            })
            .for_each(|rule| inner_rule_indices.push(rule));
        let rules = &mut types_result.rules;
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
        types_result: &mut TypesResult,
        context: Context,
        space_info: (&Namespace, &BindingMap),
    ) -> RuleIndex {
        match self {
            Self::Struct(s) => s.analyze(types_result, context, space_info),
            Self::List(list) => list.analyze(types_result, context, space_info),
            Self::Named(named) => named.analyze(types_result, context, space_info),
            Self::Primitive(primitive) => primitive.analyze(types_result, context, space_info),
        }
    }
}

impl TypeAnalyzer for StructOrList {
    fn analyze(
        &self,
        types_result: &mut TypesResult,
        context: Context,
        space_info: (&Namespace, &BindingMap),
    ) -> RuleIndex {
        match self {
            Self::Struct(s) => s.analyze(types_result, context, space_info),
            Self::List(l) => l.analyze(types_result, context, space_info),
        }
    }
}

impl TypeAnalyzer for Struct {
    fn analyze(
        &self,
        types_result: &mut TypesResult,
        context: Context,
        space_info: (&Namespace, &BindingMap),
    ) -> RuleIndex {
        let mut field_names = Vec::new();
        let mut errors = Vec::new();
        let field_rule_indices = self
            .fields
            .iter()
            .flatten()
            .inspect(|field| {
                let field_name = &field.name.name;
                if field_names.contains(&field_name) {
                    errors.push(Error::new(
                        ErrorMessage::DuplicateField(field_name.clone()),
                        field.info.to_token_range(),
                    ));
                } else {
                    field_names.push(field_name);
                }
            })
            .map(|field| {
                (
                    field.name.clone(),
                    (
                        field.analyze(types_result, context, space_info),
                        field.expr.is_some(),
                    ),
                )
            })
            .collect();
        types_result.errors.extend(errors);
        let rules = &mut types_result.rules;
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
        types_result: &mut TypesResult,
        context: Context,
        space_info: (&Namespace, &BindingMap),
    ) -> RuleIndex {
        let info = self.info.clone();
        let rule = match (
            self.type_expr
                .as_ref()
                .map(|type_expr| type_expr.analyze(types_result, Context::TypeExpr, space_info)),
            self.expr
                .as_ref()
                .map(|expr| expr.analyze(types_result, context, space_info)),
        ) {
            (Some(type_expr_index), Some(expr_index)) => {
                if !is_union(types_result, expr_index) {
                    types_result.assertions.push((type_expr_index, expr_index));
                }
                Rule {
                    type_description: TypeDescription::Rule(type_expr_index),
                    info,
                }
            }
            (Some(type_expr_index), None) => {
                if matches!(context, Context::Expr) {
                    let error = Error::new(ErrorMessage::UnassignedField, info.range.clone());
                    types_result.errors.push(error);
                }
                Rule {
                    type_description: TypeDescription::Rule(type_expr_index),
                    info,
                }
            }
            (None, Some(expr_index)) => Rule {
                type_description: TypeDescription::Rule(expr_index),
                info,
            },
            (None, None) => {
                let error = match context {
                    Context::Expr => Error::new(ErrorMessage::UnassignedField, info.range.clone()),
                    Context::TypeExpr => Error::new(ErrorMessage::UntypedField, info.range.clone()),
                };
                types_result.errors.push(error);
                Rule {
                    type_description: TypeDescription::Unknown,
                    info,
                }
            }
        };
        let rules = &mut types_result.rules;
        rules.push(rule);
        rule_index(rules)
    }
}

impl TypeAnalyzer for List {
    fn analyze(
        &self,
        types_result: &mut TypesResult,
        context: Context,
        space_info: (&Namespace, &BindingMap),
    ) -> RuleIndex {
        let inner_rule_indices = self
            .exprs
            .iter()
            .map(|expr| expr.analyze(types_result, context, space_info))
            .collect();
        let rules = &mut types_result.rules;
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
        types_result: &mut TypesResult,
        context: Context,
        space_info: (&Namespace, &BindingMap),
    ) -> RuleIndex {
        let (namespace, bindings) = space_info;
        let names = self.names();
        let (named_rule, ident) = if let Some(global_ident) = names
            .first()
            .map(Deref::deref)
            .and_then(|name| namespace.get(name))
        {
            let extended_names = names.into_iter().skip(1).map(|s| s.to_string()).collect();
            let ident = global_ident.clone().extend_multiple(extended_names);
            (
                Rule {
                    info: self.info.clone(),
                    type_description: TypeDescription::Ident(ident.clone()),
                },
                Some(ident),
            )
        } else if let Some(built_in) = names
            .first()
            // && names.len() == 1
            .and_then(|name| if names.len() == 1 { Some(name) } else { None })
            .map(Deref::deref)
            .and_then(|name| BuiltInType::into_iter().find(|built_in| built_in.as_str() == name))
        {
            (
                Rule {
                    info: self.info.clone(),
                    type_description: TypeDescription::BuiltIn(built_in),
                },
                None,
            )
        } else {
            (
                Rule {
                    info: self.info.clone(),
                    type_description: TypeDescription::Unknown,
                },
                None,
            )
        };
        let rules = &mut types_result.rules;
        rules.push(named_rule);
        let named_rule_index = rule_index(rules);
        if let Some(expr_rule_index) = self
            .expr
            .as_ref()
            .map(|expr| expr.analyze(types_result, context, space_info))
        {
            if let Some(ident) = ident {
                if matches!(bindings.get(&ident), Some(Binding::Let)) {
                    types_result.errors.push(Error::new(
                        ErrorMessage::ImmutableLet(ident.end().to_string()),
                        self.name.info.to_token_range(),
                    ));
                } else {
                    types_result
                        .assertions
                        .push((named_rule_index, expr_rule_index));
                }
            }
        };
        named_rule_index
    }
}

impl TypeAnalyzer for Primitive {
    fn analyze(
        &self,
        types_result: &mut TypesResult,
        _: Context,
        _: (&Namespace, &BindingMap),
    ) -> RuleIndex {
        let rules = &mut types_result.rules;
        rules.push(Rule {
            type_description: TypeDescription::Primitive(self.clone()),
            info: self.info().clone(),
        });
        rule_index(rules)
    }
}

fn is_union(types_result: &TypesResult, rule_index: RuleIndex) -> bool {
    let rule = types_result.rules.get(rule_index).expect("Rule must exist");
    matches!(rule.type_description, TypeDescription::Union(_))
}
