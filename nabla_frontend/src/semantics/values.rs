use crate::{
    ast::{AstInfo, Global, Ident, Let},
    eval::Value,
    semantics::{Error, ErrorMessage, Errors, SymbolTable},
    token::ToTokenRange,
    GlobalIdent, ModuleAst,
};
use std::collections::HashMap;

mod analysis;

/// Index into rule list
type RuleIndex = usize;

#[derive(Clone, Debug)]
struct Rule {
    pub value_description: ValueDescription,
    pub is_default: bool,
    pub info: AstInfo,
}

#[derive(Clone, Debug)]
enum ValueDescription {
    Union(Vec<RuleIndex>),
    Struct(HashMap<String, RuleIndex>),
    List(Vec<RuleIndex>),
    Primitive(Value),
    /// Composed(own rule, super rule)
    Composed(RuleIndex, RuleIndex),
    Ref(Ident),
    Empty,
    Unknown,
}

#[derive(Clone, Debug)]
pub struct ValuesResult {
    pub inits: Vec<Value>,
    pub symbol_table: SymbolTable,
    pub errors: Errors,
}

pub fn analyze(module_ast: &ModuleAst) -> ValuesResult {
    let mut rules = Vec::new();
    let mut rule_table: HashMap<GlobalIdent, RuleIndex> = HashMap::new();
    let mut inits: Vec<RuleIndex> = Vec::new();
    let mut lets: Vec<(&Let, RuleIndex)> = Vec::new();

    for global in module_ast.ast.globals.iter() {
        match global {
            Global::Def(d) => {
                if let Some(expr) = &d.expr {
                    analysis::analyze(expr, &mut rules);
                    if let Some(ident) = &d.name {
                        let rule_index = rules.len() - 1;
                        rule_table.insert(
                            module_ast.name.clone().extend(ident.name.clone()),
                            rule_index,
                        );
                    }
                }
            }
            Global::Let(l) => {
                if let Some(expr) = &l.expr {
                    analysis::analyze(expr, &mut rules);
                    let rule_index = rules.len() - 1;
                    if let Some(ident) = &l.name {
                        rule_table.insert(
                            module_ast.name.clone().extend(ident.name.clone()),
                            rule_index,
                        );
                    }
                    lets.push((l, rule_index));
                }
            }
            Global::Init(expr) => {
                analysis::analyze(expr, &mut rules);
                let rule_index = rules.len() - 1;
                inits.push(rule_index);
            }
            _ => {}
        }
    }
    let mut errors = Vec::new();
    let evaluated = evaluate(module_ast.name.clone(), &rules, &rule_table, &mut errors);
    for (rule_index, rule) in rules.iter().enumerate() {
        if rule.is_default {
            let value = evaluated
                .get(&rule_index)
                .expect("Rule must have been evaluated");
            if !value.is_known() {
                let error = Error::new(
                    ErrorMessage::UninitializedDefault,
                    rule.info.to_token_range(),
                );
                errors.push(error);
            }
        }
    }
    for (l, rule_index) in lets {
        let value = evaluated
            .get(&rule_index)
            .expect("Rule must have been evaluated");
        if !value.is_known() {
            let error = Error::new(ErrorMessage::UninitializedLet, l.info.to_token_range());
            errors.push(error);
        }
    }
    let symbol_table = rule_table
        .into_iter()
        .map(|(ident, rule_index)| {
            (
                ident,
                evaluated
                    .get(&rule_index)
                    .cloned()
                    .expect("Rule must have been evaluated"),
            )
        })
        .collect();
    inits.iter().skip(1).for_each(|rule_index| {
        let rule = rules.get(*rule_index).expect("Rule must exists");
        let error = Error::new(ErrorMessage::MultipleInits, rule.info.to_token_range());
        errors.push(error);
    });
    let inits = inits
        .iter()
        .map(|rule_index| {
            let value = evaluated
                .get(rule_index)
                .cloned()
                .expect("Rule must have been evaluated");
            if !value.is_known() {
                let rule = rules.get(*rule_index).expect("Rule must exists");
                let error = Error::new(ErrorMessage::UninitializedInit, rule.info.to_token_range());
                errors.push(error);
            }
            value
        })
        .collect();

    ValuesResult {
        inits,
        symbol_table,
        errors,
    }
}

fn evaluate(
    module: GlobalIdent,
    rules: &[Rule],
    rule_table: &HashMap<GlobalIdent, RuleIndex>,
    errors: &mut Vec<Error>,
) -> HashMap<RuleIndex, Value> {
    let mut stack: Vec<RuleIndex> = Vec::new();
    let mut evaluated: HashMap<RuleIndex, Value> = HashMap::with_capacity(rules.len());
    for rule_index in 0..rules.len() {
        if evaluated.contains_key(&rule_index) {
            continue;
        }
        stack.push(rule_index);
        while let Some(rule_index) = stack.pop() {
            if evaluated.contains_key(&rule_index) {
                continue;
            }

            // prevent infinite loop
            if stack.contains(&rule_index) {
                // a self reference is not necessarily illegal
                // but it cannot be fully evaluated
                evaluated.insert(rule_index, Value::Unknown);
                continue;
            }

            // push dependencies to stack
            let rule = rules.get(rule_index).expect("Rule must exist");
            match &rule.value_description {
                ValueDescription::Union(indices) => {
                    let unevaluated: Vec<_> = indices
                        .iter()
                        .filter(|index| !evaluated.contains_key(index))
                        .collect();
                    if !unevaluated.is_empty() {
                        stack.push(rule_index);
                        stack.extend(unevaluated);
                        continue;
                    }
                }
                ValueDescription::Struct(s) => {
                    let unevaluated: Vec<_> = s
                        .values()
                        .filter(|index| !evaluated.contains_key(index))
                        .collect();
                    if !unevaluated.is_empty() {
                        stack.push(rule_index);
                        stack.extend(unevaluated);
                        continue;
                    }
                }
                ValueDescription::List(indices) => {
                    let unevaluated: Vec<_> = indices
                        .iter()
                        .filter(|index| !evaluated.contains_key(index))
                        .collect();
                    if !unevaluated.is_empty() {
                        stack.push(rule_index);
                        stack.extend(unevaluated);
                        continue;
                    }
                }
                ValueDescription::Composed(i1, i2) => {
                    match (evaluated.contains_key(i1), evaluated.contains_key(i2)) {
                        (false, false) => {
                            stack.push(rule_index);
                            stack.push(*i1);
                            stack.push(*i2);
                            continue;
                        }
                        (false, true) => {
                            stack.push(rule_index);
                            stack.push(*i1);
                            continue;
                        }
                        (true, false) => {
                            stack.push(rule_index);
                            stack.push(*i2);
                            continue;
                        }
                        (true, true) => {}
                    }
                }
                ValueDescription::Ref(ident) => {
                    if ident.is_flattened() {
                        // TODO: implement lookup
                        errors.push(Error::new(
                            ErrorMessage::Unsupported("module references".to_string()),
                            ident.info.to_token_range(),
                        ));
                    }
                    if let Some(ref_index) =
                        rule_table.get(&module.clone().extend(ident.name.clone()))
                    {
                        if !evaluated.contains_key(ref_index) {
                            stack.push(rule_index);
                            stack.push(*ref_index);
                            continue;
                        }
                    }
                }
                _ => {}
            }

            // evaluate
            match &rule.value_description {
                ValueDescription::Union(_indices) => {
                    // TODO: How to deal with unions?
                    evaluated.insert(rule_index, Value::Unknown);
                }
                ValueDescription::Struct(s) => {
                    let value = Value::Struct(
                        s.iter()
                            .map(|(k, rule_index)| {
                                (
                                    k.clone(),
                                    evaluated.get(rule_index).cloned().unwrap_or(Value::Unknown),
                                )
                            })
                            .collect(),
                    );
                    evaluated.insert(rule_index, value);
                }
                ValueDescription::List(l) => {
                    let value = Value::List(
                        l.iter()
                            .map(|rule_index| {
                                evaluated.get(rule_index).cloned().unwrap_or(Value::Unknown)
                            })
                            .collect(),
                    );
                    evaluated.insert(rule_index, value);
                }
                ValueDescription::Composed(own_index, super_index) => {
                    let mut own_value = evaluated.get(own_index).cloned().unwrap_or(Value::Unknown);
                    let super_value = evaluated
                        .get(super_index)
                        .cloned()
                        .unwrap_or(Value::Unknown);
                    own_value.merge_fields(super_value);
                    evaluated.insert(rule_index, own_value);
                }
                ValueDescription::Primitive(value) => {
                    evaluated.insert(rule_index, value.clone());
                }
                ValueDescription::Ref(ident) => {
                    if let Some(ref_index) =
                        rule_table.get(&module.clone().extend(ident.name.clone()))
                    {
                        let value = evaluated
                            .get(ref_index)
                            .expect("Value must be present")
                            .clone();
                        evaluated.insert(rule_index, value);
                    } else {
                        evaluated.insert(rule_index, Value::Unknown);
                    }
                }
                ValueDescription::Empty | ValueDescription::Unknown => {
                    evaluated.insert(rule_index, Value::Unknown);
                }
            }
        }
    }
    evaluated
}
