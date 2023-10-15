use super::Error;
use crate::{ast::*, semantics::error::ErrorMessage, GlobalIdent};
use std::{array::IntoIter, collections::HashMap};

/// Index into rule list
type RuleIndex = usize;

#[derive(Clone, Debug)]
struct Rule {
    type_description: TypeDescription,
    info: AstInfo,
}

#[derive(Clone, Debug)]
enum TypeDescription {
    Union(Vec<RuleIndex>),
    Struct(HashMap<String, RuleIndex>),
    List(Vec<RuleIndex>),
    Ident(Ident),
    ValidIdent(RuleIndex),
    Primitive(Primitive),
    Rule(RuleIndex),
    Import(GlobalIdent),
    BuiltIn(BuiltInType),
    Unknown,
}

#[derive(Copy, Clone, Debug)]
enum BuiltInType {
    String,
    Number,
    Bool,
    None,
}

impl BuiltInType {
    const fn as_str(&self) -> &'static str {
        match self {
            BuiltInType::String => "String",
            BuiltInType::Number => "Number",
            BuiltInType::Bool => "Bool",
            BuiltInType::None => "None",
        }
    }

    fn into_iter() -> IntoIter<BuiltInType, 4> {
        static BUILT_INS: [BuiltInType; 4] = [
            BuiltInType::String,
            BuiltInType::Number,
            BuiltInType::Bool,
            BuiltInType::None,
        ];
        BUILT_INS.into_iter()
    }

    fn matches(&self, value: &Primitive) -> bool {
        match (self, value) {
            (BuiltInType::String, Primitive::String(_)) => true,
            (BuiltInType::Number, Primitive::Number(_)) => true,
            (BuiltInType::Bool, Primitive::Bool(_)) => true,
            _ => false,
        }
    }
}

#[derive(Clone, Debug, Default)]
struct TypeInfo<'a> {
    rules: Vec<Rule>,
    assertions: Vec<(RuleIndex, RuleIndex)>,
    idents: HashMap<&'a Ident, RuleIndex>,
    errors: Vec<Error>,
}

pub fn analyze(program: &Program) -> Vec<Error> {
    let mut type_info = TypeInfo::default();
    for global in &program.globals {
        match global {
            Global::Use(u) => analyze_use(u, &mut type_info),
            Global::Def(def) => analyze_def(def, &mut type_info),
            Global::Let(l) => analyze_let(l, &mut type_info),
            Global::Init(init) => {
                init.analyze(&mut type_info.rules, &mut type_info.assertions);
            }
            Global::Error(_) => { /* no types to check */ }
        }
    }
    replace_rules(&mut type_info);
    check_assertions(&mut type_info);
    type_info.errors
}

/// Replace all `Ident` and `Import` rules.
///
/// If the ident is defined, its rule is replaced by a `ValidIdent`-rule,
/// containing the original rule index.
/// If not, and it's a built in, it gets converted to a `BuiltIn`-rule.
/// Otherwise an error is reported and the rule type is `Unknown`.
///
/// Imports are looked up and replaced by their rule type, if present,
/// and an `Unknown`-rule otherwise.
/// Currently the lookup is not implemented, so every import is `Unknown`.
fn replace_rules(type_info: &mut TypeInfo) {
    for rule in type_info.rules.iter_mut() {
        let replacement = match &rule.type_description {
            TypeDescription::Ident(ident) => {
                let rule_index = type_info.idents.get(ident).map(|index| *index);
                if let Some(rule_index) = rule_index {
                    Some(TypeDescription::ValidIdent(rule_index))
                } else if let Some(built_in) =
                    BuiltInType::into_iter().find(|built_in| built_in.as_str() == &ident.name)
                {
                    Some(TypeDescription::BuiltIn(built_in))
                } else {
                    type_info.errors.push(Error::new(
                        ErrorMessage::UndefinedIdent(ident.name.clone()),
                        ident.info.range.clone(),
                    ));
                    Some(TypeDescription::Unknown)
                }
            }
            TypeDescription::Import(_) => {
                // currently not implementing use
                Some(TypeDescription::Unknown)
            }
            _ => None,
        };
        if let Some(replacement) = replacement {
            rule.type_description = replacement;
        }
    }
}

fn check_assertions(type_info: &mut TypeInfo) {
    fn check_union(rules: &[Rule], expected: &[RuleIndex], actual_rule: &Rule) -> Vec<Error> {
        for expected_rule in expected
            .into_iter()
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
                    .into_iter()
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
                ErrorMessage::ValueMismatch(
                    expected.as_str().to_string(),
                    actual.as_str().to_string(),
                ),
                actual.info().range.clone(),
            )]
        }
    }

    fn check_value(expected: &BuiltInType, actual: &Primitive) -> Vec<Error> {
        if expected.matches(actual) {
            Vec::new()
        } else {
            vec![Error::new(
                ErrorMessage::ValueMismatch(
                    expected.as_str().to_string(),
                    actual.as_str().to_string(),
                ),
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
            (
                TypeDescription::BuiltIn(BuiltInType::None),
                TypeDescription::BuiltIn(BuiltInType::None),
            ) => Vec::new(),
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

fn analyze_def<'a>(def: &'a Def, type_info: &mut TypeInfo<'a>) {
    let TypeInfo {
        ref mut rules,
        ref mut assertions,
        ref mut idents,
        ref mut errors,
    } = type_info;
    if let Some(rule_index) = def
        .expr
        .as_ref()
        .map(|expr| expr.analyze(rules, assertions))
    {
        if let Some(name) = &def.name {
            use std::collections::hash_map::Entry;
            match idents.entry(name) {
                Entry::Vacant(entry) => {
                    entry.insert(rule_index);
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
}

fn analyze_let<'a>(l: &'a Let, type_info: &mut TypeInfo<'a>) {
    let TypeInfo {
        ref mut rules,
        ref mut assertions,
        ref mut idents,
        ref mut errors,
    } = type_info;
    if let Some(rule_index) = l.expr.as_ref().map(|expr| expr.analyze(rules, assertions)) {
        if let Some(name) = &l.name {
            use std::collections::hash_map::Entry;
            match idents.entry(name) {
                Entry::Vacant(entry) => {
                    entry.insert(rule_index);
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
}

fn analyze_use<'a>(u: &'a Use, type_info: &mut TypeInfo<'a>) {
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
            let import_rule_index = rule_index(&rules);
            let name = item
                .alias
                .as_ref()
                .and_then(|alias| alias.name.as_ref())
                .and_then(|alias_name| match alias_name {
                    AliasName::Ident(ident) => Some(ident),
                    AliasName::String(_) => None,
                })
                .unwrap_or_else(|| &item.name);
            use std::collections::hash_map::Entry;
            match idents.entry(&name) {
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

fn rule_index(rules: &[Rule]) -> RuleIndex {
    rules.len() - 1
}

trait TypeAnalyzer {
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
            Expr::Union(union) => union.analyze(rules, assertions),
            Expr::Single(single) => single.analyze(rules, assertions),
            Expr::Error(info) => {
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
            Single::Struct(s) => s.analyze(rules, assertions),
            Single::List(list) => list.analyze(rules, assertions),
            Single::Named(named) => named.analyze(rules, assertions),
            Single::Primitive(primitive) => primitive.analyze(rules, assertions),
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
            StructOrList::Struct(s) => s.analyze(rules, assertions),
            StructOrList::List(l) => l.analyze(rules, assertions),
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
        let named_rule_index = rule_index(&rules);
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
