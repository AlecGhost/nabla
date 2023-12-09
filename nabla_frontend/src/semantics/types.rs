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

pub const STRING: &str = "String";
pub const NUMBER: &str = "Number";
pub const BOOL: &str = "Bool";

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
    Struct(HashMap<String, (RuleIndex, bool)>),
    List(Vec<RuleIndex>),
    Ident(Ident),
    ValidIdent(RuleIndex),
    Primitive(Primitive),
    Rule(RuleIndex),
    Import(GlobalIdent),
    BuiltIn(BuiltInType),
    Unknown,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum BuiltInType {
    String,
    Number,
    Bool,
}

impl BuiltInType {
    const fn as_str(&self) -> &'static str {
        match self {
            Self::String => STRING,
            Self::Number => NUMBER,
            Self::Bool => BOOL,
        }
    }

    fn into_iter() -> IntoIter<Self, 3> {
        static BUILT_INS: [BuiltInType; 3] =
            [BuiltInType::String, BuiltInType::Number, BuiltInType::Bool];
        BUILT_INS.into_iter()
    }

    const fn matches(&self, value: &Primitive) -> bool {
        matches!(
            (self, value),
            (Self::String, Primitive::String(_))
                | (Self::Number, Primitive::Number(_))
                | (Self::Bool, Primitive::Bool(_))
        )
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
                init.analyze(&mut type_info);
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
                let rule_index = type_info.idents.get(ident).copied();
                if let Some(rule_index) = rule_index {
                    Some(TypeDescription::ValidIdent(rule_index))
                } else if let Some(built_in) =
                    BuiltInType::into_iter().find(|built_in| built_in.as_str() == ident.name)
                {
                    Some(TypeDescription::BuiltIn(built_in))
                } else {
                    // TODO: check imports
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
