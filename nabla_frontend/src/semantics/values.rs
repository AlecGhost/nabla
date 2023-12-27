use crate::{
    ast::{AstInfo, Global, Ident, Let, Program},
    eval::Value,
    semantics::{Error, ErrorMessage},
};
use std::collections::HashMap;

mod value_analysis;

/// Index into rule list
type RuleIndex = usize;

pub type SymbolTable = HashMap<Ident, Value>;

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

pub fn analyze(program: &Program) -> (Vec<Value>, SymbolTable, Vec<Error>) {
    let mut rules = Vec::new();
    let mut rule_table: HashMap<Ident, RuleIndex> = HashMap::new();
    let mut inits: Vec<RuleIndex> = Vec::new();
    let mut lets: Vec<(&Let, RuleIndex)> = Vec::new();

    for global in program.globals.iter() {
        match global {
            Global::Def(d) => {
                if let Some(expr) = &d.expr {
                    value_analysis::analyze(expr, &mut rules);
                    if let Some(ident) = &d.name {
                        let rule_index = rules.len() - 1;
                        rule_table.insert(ident.clone(), rule_index);
                    }
                }
            }
            Global::Let(l) => {
                if let Some(expr) = &l.expr {
                    value_analysis::analyze(expr, &mut rules);
                    let rule_index = rules.len() - 1;
                    if let Some(ident) = &l.name {
                        rule_table.insert(ident.clone(), rule_index);
                    }
                    lets.push((l, rule_index));
                }
            }
            Global::Init(expr) => {
                value_analysis::analyze(expr, &mut rules);
                let rule_index = rules.len() - 1;
                inits.push(rule_index);
            }
            _ => {}
        }
    }
    let mut errors = Vec::new();
    let evaluated = evaluate(&rules, &rule_table, &mut errors);
    for (rule_index, rule) in rules.iter().enumerate() {
        if rule.is_default {
            let value = evaluated
                .get(&rule_index)
                .expect("Rule must have been evaluated");
            if !value.is_known() {
                let error = Error::new(ErrorMessage::UninitializedDefault, rule.info.range.clone());
                errors.push(error);
            }
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
    let inits = inits
        .iter()
        .map(|rule_index| {
            evaluated
                .get(rule_index)
                .cloned()
                .expect("Rule must have been evaluated")
        })
        .collect();
    (inits, symbol_table, errors)
}

fn evaluate(
    rules: &[Rule],
    rule_table: &HashMap<Ident, RuleIndex>,
    errors: &mut Vec<Error>,
) -> HashMap<RuleIndex, Value> {
    let mut stack: Vec<RuleIndex> = Vec::new();
    let mut evaluated: HashMap<RuleIndex, Value> = HashMap::with_capacity(rules.len());
    let mut index = 0;
    while evaluated.len() != rules.len() {
        if let Some(rule_index) = stack.pop() {
            if evaluated.contains_key(&rule_index) {
                continue;
            }
            if stack.contains(&rule_index) {
                // recursive call
                let rule = rules.get(rule_index).expect("Rule must exist");
                let error = Error::new(ErrorMessage::RecursiveInit, rule.info.range.clone());
                errors.push(error);
                evaluated.insert(rule_index, Value::Unknown);
            }
            let rule = rules.get(rule_index).expect("Rule must exist");
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
                    let ref_index = rule_table
                        .get(ident)
                        .expect("Rule table must contain ident");
                    let value = evaluated
                        .get(ref_index)
                        .expect("Value must be present")
                        .clone();
                    evaluated.insert(rule_index, value);
                }
                ValueDescription::Empty | ValueDescription::Unknown => {
                    evaluated.insert(rule_index, Value::Unknown);
                }
            }
        } else if evaluated.contains_key(&index) {
            index += 1;
            continue;
        } else {
            let rule = rules.get(index).expect("Rule must exist");
            match &rule.value_description {
                ValueDescription::Union(indices) => {
                    stack.push(index);
                    stack.extend(indices);
                }
                ValueDescription::Struct(s) => {
                    stack.push(index);
                    stack.extend(s.values());
                }
                ValueDescription::List(indices) => {
                    stack.push(index);
                    stack.extend(indices);
                }
                ValueDescription::Primitive(value) => {
                    evaluated.insert(index, value.clone());
                }
                ValueDescription::Composed(i1, i2) => {
                    stack.push(index);
                    stack.push(*i1);
                    stack.push(*i2);
                }
                ValueDescription::Ref(ident) => match rule_table.get(ident) {
                    Some(rule_index) => {
                        stack.push(index);
                        stack.push(*rule_index);
                    }
                    None => {
                        evaluated.insert(index, Value::Unknown);
                    }
                },
                ValueDescription::Empty | ValueDescription::Unknown => {
                    evaluated.insert(index, Value::Unknown);
                }
            }
            index += 1;
        }
    }
    evaluated
}