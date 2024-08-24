use std::ops::Deref;

use crate::{
    ast::*,
    semantics::{
        error::{Error, ErrorMessage},
        types::{BuiltInType, Context, Rule, RuleIndex, TypeDescription, TypeInfo},
        Namespace,
    },
    token::ToTokenRange,
};

pub(super) fn analyze_def(
    def: &Def,
    type_info: &mut TypeInfo,
    namespace: &Namespace,
) -> Option<RuleIndex> {
    analyze_binding(
        def.name.as_ref(),
        def.type_expr.as_ref(),
        def.expr.as_ref(),
        type_info,
        Context::TypeExpr,
        namespace,
    )
}

pub(super) fn analyze_let(
    l: &Let,
    type_info: &mut TypeInfo,
    namespace: &Namespace,
) -> Option<RuleIndex> {
    analyze_binding(
        l.name.as_ref(),
        l.type_expr.as_ref(),
        l.expr.as_ref(),
        type_info,
        Context::Expr,
        namespace,
    )
}

fn analyze_binding<'a>(
    name: Option<&'a Ident>,
    type_expr: Option<&'a Expr>,
    expr: Option<&'a Expr>,
    type_info: &mut TypeInfo,
    context: Context,
    namespace: &Namespace,
) -> Option<RuleIndex> {
    fn check_self_reference(ident: &Ident, expr: Option<&Expr>) -> bool {
        matches!(expr, Some(Expr::Single(Single::Named(Named { name, .. }))) if ident == name)
    }

    if let Some(name) = name {
        if check_self_reference(name, type_expr) || check_self_reference(name, expr) {
            type_info.errors.push(Error::new(
                ErrorMessage::SelfReference(name.name.clone()),
                name.info.to_token_range(),
            ));
            return None;
        }
    }
    match (
        type_expr
            .as_ref()
            .map(|type_expr| type_expr.analyze(type_info, Context::TypeExpr, namespace)),
        expr.as_ref()
            .map(|expr| expr.analyze(type_info, context, namespace)),
    ) {
        (Some(type_expr_index), Some(expr_index)) => {
            if !(matches!(context, Context::Expr) && is_union(type_info, expr_index)) {
                type_info.assertions.push((type_expr_index, expr_index));
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
        type_info: &mut TypeInfo,
        context: Context,
        namespace: &Namespace,
    ) -> RuleIndex;
}

impl TypeAnalyzer for Expr {
    fn analyze(
        &self,
        type_info: &mut TypeInfo,
        context: Context,
        namespace: &Namespace,
    ) -> RuleIndex {
        let rules = &mut type_info.rules;
        match self {
            Self::Union(union) => union.analyze(type_info, context, namespace),
            Self::Single(single) => single.analyze(type_info, context, namespace),
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
        type_info: &mut TypeInfo,
        context: Context,
        namespace: &Namespace,
    ) -> RuleIndex {
        let mut inner_rule_indices = Vec::with_capacity(self.alternatives.len() + 1);
        inner_rule_indices.push(self.single.analyze(type_info, context, namespace));
        self.alternatives
            .iter()
            .flat_map(|alternative| {
                alternative
                    .single
                    .as_ref()
                    .map(|single| single.analyze(type_info, context, namespace))
            })
            .for_each(|rule| inner_rule_indices.push(rule));
        let rules = &mut type_info.rules;
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
        type_info: &mut TypeInfo,
        context: Context,
        namespace: &Namespace,
    ) -> RuleIndex {
        match self {
            Self::Struct(s) => s.analyze(type_info, context, namespace),
            Self::List(list) => list.analyze(type_info, context, namespace),
            Self::Named(named) => named.analyze(type_info, context, namespace),
            Self::Primitive(primitive) => primitive.analyze(type_info, context, namespace),
        }
    }
}

impl TypeAnalyzer for StructOrList {
    fn analyze(
        &self,
        type_info: &mut TypeInfo,
        context: Context,
        namespace: &Namespace,
    ) -> RuleIndex {
        match self {
            Self::Struct(s) => s.analyze(type_info, context, namespace),
            Self::List(l) => l.analyze(type_info, context, namespace),
        }
    }
}

impl TypeAnalyzer for Struct {
    fn analyze(
        &self,
        type_info: &mut TypeInfo,
        context: Context,
        namespace: &Namespace,
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
                        field.analyze(type_info, context, namespace),
                        field.expr.is_some(),
                    ),
                )
            })
            .collect();
        type_info.errors.extend(errors);
        let rules = &mut type_info.rules;
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
        type_info: &mut TypeInfo,
        context: Context,
        namespace: &Namespace,
    ) -> RuleIndex {
        let info = self.info.clone();
        let rule = match (
            self.type_expr
                .as_ref()
                .map(|type_expr| type_expr.analyze(type_info, Context::TypeExpr, namespace)),
            self.expr
                .as_ref()
                .map(|expr| expr.analyze(type_info, context, namespace)),
        ) {
            (Some(type_expr_index), Some(expr_index)) => {
                if !is_union(type_info, expr_index) {
                    type_info.assertions.push((type_expr_index, expr_index));
                }
                Rule {
                    type_description: TypeDescription::Rule(type_expr_index),
                    info,
                }
            }
            (Some(type_expr_index), None) => {
                if matches!(context, Context::Expr) {
                    let error = Error::new(ErrorMessage::UnassignedField, info.range.clone());
                    type_info.errors.push(error);
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
                type_info.errors.push(error);
                Rule {
                    type_description: TypeDescription::Unknown,
                    info,
                }
            }
        };
        let rules = &mut type_info.rules;
        rules.push(rule);
        rule_index(rules)
    }
}

impl TypeAnalyzer for List {
    fn analyze(
        &self,
        type_info: &mut TypeInfo,
        context: Context,
        namespace: &Namespace,
    ) -> RuleIndex {
        let inner_rule_indices = self
            .exprs
            .iter()
            .map(|expr| expr.analyze(type_info, context, namespace))
            .collect();
        let rules = &mut type_info.rules;
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
        type_info: &mut TypeInfo,
        context: Context,
        namespace: &Namespace,
    ) -> RuleIndex {
        let names = self.names();
        let (named_rule, is_valid) = if let Some(global_ident) = names
            .first()
            .map(Deref::deref)
            .and_then(|name| namespace.get(name))
        {
            let extended_names = names.into_iter().skip(1).map(|s| s.to_string()).collect();
            (
                Rule {
                    info: self.info.clone(),
                    type_description: TypeDescription::Ident(
                        global_ident.clone().extend_multiple(extended_names),
                    ),
                },
                true,
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
                false,
            )
        } else {
            (
                Rule {
                    info: self.info.clone(),
                    type_description: TypeDescription::Unknown,
                },
                false,
            )
        };
        let rules = &mut type_info.rules;
        rules.push(named_rule);
        let named_rule_index = rule_index(rules);
        if let Some(expr_rule_index) = self
            .expr
            .as_ref()
            .map(|expr| expr.analyze(type_info, context, namespace))
        {
            if is_valid {
                type_info
                    .assertions
                    .push((named_rule_index, expr_rule_index));
            }
        };
        named_rule_index
    }
}

impl TypeAnalyzer for Primitive {
    fn analyze(&self, type_info: &mut TypeInfo, _: Context, _: &Namespace) -> RuleIndex {
        let rules = &mut type_info.rules;
        rules.push(Rule {
            type_description: TypeDescription::Primitive(self.clone()),
            info: self.info().clone(),
        });
        rule_index(rules)
    }
}

fn is_union(type_info: &TypeInfo, rule_index: RuleIndex) -> bool {
    let rule = type_info.rules.get(rule_index).expect("Rule must exist");
    matches!(rule.type_description, TypeDescription::Union(_))
}
