use crate::{
    ast::Ident,
    semantics::{
        error::ErrorMessage,
        types::{BuiltInType, Primitive, Rule, RuleIndex, TypeDescription, TypeInfo},
        Error,
    },
    token::{ToTokenRange, TokenRange},
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

fn check_union(rules: &[Rule], expected: &[RuleIndex], actual: &[RuleIndex]) -> Vec<Error> {
    let expected_rules: Vec<&Rule> = expected
        .iter()
        .map(|rule_index| rules.get(*rule_index).expect("Rule must exist"))
        .collect();
    actual
        .iter()
        .map(|rule_index| rules.get(*rule_index).expect("Rule must exist"))
        .filter(|actual_rule| {
            !expected_rules
                .iter()
                .any(|expected_rule| check_rules(rules, expected_rule, actual_rule).is_empty())
        })
        .map(|actual_rule| {
            Error::new(
                ErrorMessage::TypeMismatch,
                actual_rule.info.to_token_range(),
            )
        })
        .collect()
}

fn check_in_union(rules: &[Rule], expected: &[RuleIndex], actual_rule: &Rule) -> Vec<Error> {
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
        actual_rule.info.to_token_range(),
    )]
}

fn check_struct(
    rules: &[Rule],
    expected: &HashMap<Ident, (RuleIndex, bool)>,
    actual_rule: &Rule,
    actual: &HashMap<Ident, (RuleIndex, bool)>,
) -> Vec<Error> {
    let mut errors = Vec::new();
    for (field, (expected_index, has_default)) in expected {
        if let Some((actual_index, _)) = actual.get(field) {
            let expected_rule = rules.get(*expected_index).expect("Rule must exist");
            let actual_rule = rules.get(*actual_index).expect("Rule must exist");
            errors.extend(check_rules(rules, expected_rule, actual_rule));
        } else if !has_default {
            errors.push(Error::new(
                ErrorMessage::MissingField(field.name.clone()),
                actual_rule.info.range.clone(),
            ));
        }
    }
    for field in actual.keys() {
        if !expected.contains_key(field) {
            errors.push(Error::new(
                ErrorMessage::UnexpecedField(field.name.clone()),
                field.info.range.clone(),
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
                actual_indices
                    .iter()
                    .map(|index| rules.get(*index).expect("Rule must exist"))
                    .map(|rule| &rule.info.range)
                    .cloned()
                    .map(|range| Error::new(ErrorMessage::UnexpecedListElement, range))
                    .collect()
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
        // TODO: Check outside of assertions
        // TODO: Should this be legal (value-type-duality)?
        _ => expected_indices
            .iter()
            .skip(1)
            .map(|index| rules.get(*index).expect("Rule must exist"))
            .map(|rule| &rule.info.range)
            .cloned()
            .map(|range| Error::new(ErrorMessage::MultipleListTypes, range))
            .collect(),
    }
}

fn check_built_in(expected: &BuiltInType, actual: &BuiltInType, range: TokenRange) -> Vec<Error> {
    if expected == actual {
        Vec::new()
    } else {
        vec![Error::new(ErrorMessage::TypeMismatch, range)]
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
        // sort out types that were already replaced by `lookup_imports`, `validate_idents` or extracted
        (TypeDescription::Ident(_), _)
        | (TypeDescription::ValidIdent(_), _)
        | (TypeDescription::Rule(_), _)
        | (TypeDescription::Import(_), _)
        | (_, TypeDescription::Ident(_))
        | (_, TypeDescription::ValidIdent(_))
        | (_, TypeDescription::Rule(_))
        | (_, TypeDescription::Import(_)) => panic!("Unexpected type description"),
        // union
        (TypeDescription::Union(expected), TypeDescription::Union(actual)) => {
            check_union(rules, expected, actual)
        }
        (TypeDescription::Union(union), _) => check_in_union(rules, union, actual_rule),
        // unknown
        (TypeDescription::Unknown, _) => vec![Error::new(
            ErrorMessage::UnknownType,
            expected_rule.info.to_token_range(),
        )],
        // built in
        (TypeDescription::BuiltIn(expected), TypeDescription::Primitive(actual)) => {
            check_value(expected, actual)
        }
        (TypeDescription::BuiltIn(expected), TypeDescription::BuiltIn(actual)) => {
            check_built_in(expected, actual, actual_rule.info.to_token_range())
        }
        // struct
        (TypeDescription::Struct(expected), TypeDescription::Struct(actual)) => {
            check_struct(rules, expected, actual_rule, actual)
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
            actual_rule.info.to_token_range(),
        )],
    }
}
