use crate::{
    ast::*,
    semantics::{
        error::{Error, ErrorMessage},
        types::analysis::TypeAnalyzer,
    },
    GlobalIdent,
};
use std::{array::IntoIter, collections::HashMap};

mod analysis;
mod assertions;

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
            Global::Use(u) => analysis::analyze_use(u, &mut type_info),
            Global::Def(def) => analysis::analyze_def(def, &mut type_info),
            Global::Let(l) => analysis::analyze_let(l, &mut type_info),
            Global::Init(init) => {
                init.analyze(&mut type_info.rules, &mut type_info.assertions);
            }
            Global::Error(_) => { /* no types to check */ }
        }
    }
    replace_rules(&mut type_info);
    assertions::check(&mut type_info);
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
