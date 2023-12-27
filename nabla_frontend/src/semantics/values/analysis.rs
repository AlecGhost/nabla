use crate::{
    ast::{Expr, List, Named, Single, Struct, StructField, StructOrList, Union, UnionAlternative},
    eval::Eval,
    semantics::values::{Rule, RuleIndex, ValueDescription},
};

pub(super) fn analyze(expr: &Expr, rules: &mut Vec<Rule>) {
    expr.analyze(rules);
}

trait ValueAnalyzer {
    fn analyze(&self, rules: &mut Vec<Rule>) -> RuleIndex;
}

impl ValueAnalyzer for Expr {
    fn analyze(&self, rules: &mut Vec<Rule>) -> RuleIndex {
        match self {
            Self::Single(single) => single.analyze(rules),
            Self::Union(union) => union.analyze(rules),
            _ => {
                let value_description = ValueDescription::Unknown;
                let rule = Rule {
                    value_description,
                    is_default: false,
                    info: self.info().clone(),
                };
                rules.push(rule);
                rule_index(rules)
            }
        }
    }
}

impl ValueAnalyzer for Union {
    fn analyze(&self, rules: &mut Vec<Rule>) -> RuleIndex {
        let mut indices = Vec::new();
        indices.push(self.single.analyze(rules));
        indices.extend(
            self.alternatives
                .iter()
                .map(|alternative| alternative.analyze(rules)),
        );
        let value_description = ValueDescription::Union(indices);
        let rule = Rule {
            value_description,
            is_default: false,
            info: self.info.clone(),
        };
        rules.push(rule);
        rule_index(rules)
    }
}

impl ValueAnalyzer for UnionAlternative {
    fn analyze(&self, rules: &mut Vec<Rule>) -> RuleIndex {
        self.single
            .as_ref()
            .map(|single| single.analyze(rules))
            .unwrap_or_else(|| {
                let value_description = ValueDescription::Unknown;
                let rule = Rule {
                    value_description,
                    is_default: false,
                    info: self.info.clone(),
                };
                rules.push(rule);
                rule_index(rules)
            })
    }
}

impl ValueAnalyzer for Single {
    fn analyze(&self, rules: &mut Vec<Rule>) -> RuleIndex {
        match self {
            Self::Struct(s) => s.analyze(rules),
            Self::List(l) => l.analyze(rules),
            Self::Named(n) => n.analyze(rules),
            Self::Primitive(p) => {
                let value_description = ValueDescription::Primitive(p.eval());
                let rule = Rule {
                    value_description,
                    is_default: false,
                    info: p.info().clone(),
                };
                rules.push(rule);
                rule_index(rules)
            }
        }
    }
}

impl ValueAnalyzer for Named {
    fn analyze(&self, rules: &mut Vec<Rule>) -> RuleIndex {
        let ident = self.flatten_name();
        let ident_rule_index = {
            let info = ident.info.clone();
            let value_description = ValueDescription::Ref(ident);
            let rule = Rule {
                value_description,
                is_default: false,
                info,
            };
            rules.push(rule);
            rule_index(rules)
        };
        self.expr
            .as_ref()
            .map(|s_or_l| match s_or_l {
                StructOrList::Struct(s) => s.analyze(rules),
                StructOrList::List(l) => l.analyze(rules),
            })
            .map_or(ident_rule_index, |expr_rule_index| {
                let value_description =
                    ValueDescription::Composed(expr_rule_index, ident_rule_index);
                let rule = Rule {
                    value_description,
                    is_default: false,
                    info: self.info.clone(),
                };
                rules.push(rule);
                rule_index(rules)
            })
    }
}

impl ValueAnalyzer for Struct {
    fn analyze(&self, rules: &mut Vec<Rule>) -> RuleIndex {
        let map = self
            .fields
            .iter()
            .flatten()
            .map(|field| {
                let name = field.name.name.clone();
                let index = field.analyze(rules);
                (name, index)
            })
            .collect();
        let value_description = ValueDescription::Struct(map);
        let rule = Rule {
            value_description,
            is_default: false,
            info: self.info.clone(),
        };
        rules.push(rule);
        rule_index(rules)
    }
}

impl ValueAnalyzer for List {
    fn analyze(&self, rules: &mut Vec<Rule>) -> RuleIndex {
        let indices = self.exprs.iter().map(|expr| expr.analyze(rules)).collect();
        let value_description = ValueDescription::List(indices);
        let rule = Rule {
            value_description,
            is_default: false,
            info: self.info.clone(),
        };
        rules.push(rule);
        rule_index(rules)
    }
}

impl ValueAnalyzer for StructField {
    fn analyze(&self, rules: &mut Vec<Rule>) -> RuleIndex {
        let type_expr_index = self
            .type_expr
            .as_ref()
            .map(|type_expr| type_expr.analyze(rules));
        let expr_index = self.expr.as_ref().map(|expr| expr.analyze(rules));
        match (type_expr_index, expr_index) {
            (Some(type_expr_index), Some(expr_index)) => {
                let value_description = ValueDescription::Composed(expr_index, type_expr_index);
                let rule = Rule {
                    value_description,
                    is_default: true,
                    info: self.info.clone(),
                };
                rules.push(rule);
                rule_index(rules)
            }
            (None, Some(expr_index)) => {
                let rule = rules.get_mut(expr_index).expect("Rule must exist");
                rule.is_default = true;
                expr_index
            }
            (Some(type_expr_index), None) => {
                let rule = rules.get_mut(type_expr_index).expect("Rule must exist");
                rule.is_default = false;
                type_expr_index
            }
            _ => {
                let value_description = ValueDescription::Empty;
                let rule = Rule {
                    value_description,
                    is_default: false,
                    info: self.info.clone(),
                };
                rules.push(rule);
                rule_index(rules)
            }
        }
    }
}

#[inline]
const fn rule_index(rules: &[Rule]) -> RuleIndex {
    rules.len() - 1
}
