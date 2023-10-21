use crate::semantics::{
    error::ErrorMessage,
    types::{BuiltInType, Primitive, Rule, RuleIndex, TypeDescription, TypeInfo},
    Error,
};
use std::collections::HashMap;

pub(super) fn check(type_info: &mut TypeInfo) {
    let TypeInfo {
        ref rules,
        ref assertions,
        ref mut errors,
        ..
    } = type_info;
    for (expected_index, actual_index) in assertions {
        let expected_rule = rules.get(*expected_index).expect("Rule must exist");
        let actual_rule = rules.get(*actual_index).expect("Rule must exist");
        errors.extend(check_rules(rules, expected_rule, actual_rule));
    }
}

fn check_union(rules: &[Rule], expected: &[RuleIndex], actual_rule: &Rule) -> Vec<Error> {
    for expected_rule in expected
        .iter()
        .map(|rule_index| rules.get(*rule_index).expect("Rule must exist"))
    {
        if check_rules(rules, expected_rule, actual_rule).is_empty() {
            return Vec::new();
        }
    }
    vec![Error::new(
        ErrorMessage::TypeMismatch,
        actual_rule.info.range.clone(),
    )]
}

fn check_struct(
    rules: &[Rule],
    expected: &HashMap<String, RuleIndex>,
    actual: &HashMap<String, RuleIndex>,
) -> Vec<Error> {
    let mut errors = Vec::new();
    for (field_name, expected_index) in expected {
        if let Some(actual_index) = actual.get(field_name) {
            let expected_rule = rules.get(*expected_index).expect("Rule must exist");
            let actual_rule = rules.get(*actual_index).expect("Rule must exist");
            errors.extend(check_rules(rules, expected_rule, actual_rule));
        } else {
            // TODO: find out range
            errors.push(Error::new(
                ErrorMessage::MissingField(field_name.clone()),
                0..0,
            ));
        }
    }
    for field_name in actual.keys() {
        if !expected.contains_key(field_name) {
            // TODO: find out range
            errors.push(Error::new(
                ErrorMessage::UnexpecedField(field_name.clone()),
                0..0,
            ));
        }
    }
    errors
}

fn check_list(
    rules: &[Rule],
    expected_indices: &[RuleIndex],
    actual_indices: &[RuleIndex],
) -> Vec<Error> {
    match expected_indices.len() {
        0 => {
            if actual_indices.is_empty() {
                Vec::new()
            } else {
                vec![Error::new(
                    ErrorMessage::UnexpecedListElement,
                    // TODO: find out range
                    0..0,
                )]
            }
        }
        1 => {
            let expected_rule = rules.get(expected_indices[0]).expect("Rule must exist");
            actual_indices
                .iter()
                .map(|rule_index| rules.get(*rule_index).expect("Rule must exist"))
                .flat_map(|actual_rule| check_rules(rules, expected_rule, actual_rule))
                .collect()
        }
        // TODO: find out range
        _ => vec![Error::new(ErrorMessage::MultipleListTypes, 0..0)],
    }
}

fn check_primitive(expected: &Primitive, actual: &Primitive) -> Vec<Error> {
    if expected == actual {
        Vec::new()
    } else {
        vec![Error::new(
            ErrorMessage::ValueMismatch(expected.as_str().to_string(), actual.as_str().to_string()),
            actual.info().range.clone(),
        )]
    }
}

fn check_value(expected: &BuiltInType, actual: &Primitive) -> Vec<Error> {
    if expected.matches(actual) {
        Vec::new()
    } else {
        vec![Error::new(
            ErrorMessage::ValueMismatch(expected.as_str().to_string(), actual.as_str().to_string()),
            actual.info().range.clone(),
        )]
    }
}

fn extract_type_description<'a>(
    rules: &'a [Rule],
    type_description: &'a TypeDescription,
) -> &'a TypeDescription {
    match type_description {
        TypeDescription::Union(_)
        | TypeDescription::Struct(_)
        | TypeDescription::List(_)
        | TypeDescription::Primitive(_)
        | TypeDescription::BuiltIn(_)
        | TypeDescription::Unknown => type_description, // no need to extract
        TypeDescription::ValidIdent(rule_index) | TypeDescription::Rule(rule_index) => {
            let rule = rules.get(*rule_index).expect("Rule must exist");
            extract_type_description(rules, &rule.type_description)
        }
        _ => panic!("Unexpected type description"),
    }
}

fn check_rules(rules: &[Rule], expected_rule: &Rule, actual_rule: &Rule) -> Vec<Error> {
    match (
        extract_type_description(rules, &expected_rule.type_description),
        extract_type_description(rules, &actual_rule.type_description),
    ) {
        // sort out types that were already replaced by `replace_rules` or extracted
        (TypeDescription::Ident(_), _)
        | (TypeDescription::ValidIdent(_), _)
        | (TypeDescription::Rule(_), _)
        | (TypeDescription::Import(_), _)
        | (_, TypeDescription::Ident(_))
        | (_, TypeDescription::ValidIdent(_))
        | (_, TypeDescription::Rule(_))
        | (_, TypeDescription::Import(_)) => panic!("Unexpected type description"),
        // union
        (_, TypeDescription::Union(_)) => {
            vec![Error::new(
                ErrorMessage::UnionInInit,
                actual_rule.info.range.clone(),
            )]
        }
        (TypeDescription::Union(union), _) => check_union(rules, union, actual_rule),
        // unknown
        (TypeDescription::Unknown, _) => vec![Error::new(
            ErrorMessage::UnknownType,
            expected_rule.info.range.clone(),
        )],
        // built in
        (TypeDescription::BuiltIn(expected), TypeDescription::Primitive(actual)) => {
            check_value(expected, actual)
        }
        // struct
        (TypeDescription::Struct(expected), TypeDescription::Struct(actual)) => {
            check_struct(rules, expected, actual)
        }
        // list
        (TypeDescription::List(expected), TypeDescription::List(actual)) => {
            check_list(rules, expected, actual)
        }
        // primitive
        (TypeDescription::Primitive(expected), TypeDescription::Primitive(actual)) => {
            check_primitive(expected, actual)
        }
        (_expected, _actual) => vec![Error::new(
            ErrorMessage::TypeMismatch,
            actual_rule.info.range.clone(),
        )],
    }
}
