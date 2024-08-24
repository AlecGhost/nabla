use crate::{
    ast::*,
    semantics::{error::Error, types::analysis::TypeAnalyzer, Namespace},
    GlobalIdent, ModuleAst,
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
pub struct Rule {
    pub type_description: TypeDescription,
    pub info: AstInfo,
}

#[derive(Clone, Debug)]
pub enum TypeDescription {
    Union(Vec<RuleIndex>),
    Struct(HashMap<Ident, (RuleIndex, bool)>),
    List(Vec<RuleIndex>),
    Ident(GlobalIdent),
    ValidIdent(RuleIndex),
    Primitive(Primitive),
    Rule(RuleIndex),
    BuiltIn(BuiltInType),
    Unknown,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BuiltInType {
    String,
    Number,
    Bool,
}

impl BuiltInType {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::String => STRING,
            Self::Number => NUMBER,
            Self::Bool => BOOL,
        }
    }

    pub fn into_iter() -> IntoIter<Self, 3> {
        static BUILT_INS: [BuiltInType; 3] =
            [BuiltInType::String, BuiltInType::Number, BuiltInType::Bool];
        BUILT_INS.into_iter()
    }

    pub const fn matches(&self, value: &Primitive) -> bool {
        matches!(
            (self, value),
            (Self::String, Primitive::String(_))
                | (Self::Number, Primitive::Number(_))
                | (Self::Bool, Primitive::Bool(_))
        )
    }
}

#[derive(Clone, Copy, Debug)]
enum Context {
    Expr,
    TypeExpr,
}

#[derive(Clone, Debug, Default)]
pub struct TypeInfo {
    pub rules: Vec<Rule>,
    pub assertions: Vec<(RuleIndex, RuleIndex)>,
    pub errors: Vec<Error>,
}

pub fn analyze(module_ast: &ModuleAst, namespace: &Namespace) -> TypeInfo {
    let mut type_info = TypeInfo::default();
    let ident_rules: HashMap<GlobalIdent, RuleIndex> = module_ast
        .ast
        .globals
        .iter()
        .flat_map(|global| {
            match global {
                Global::Def(def) => {
                    analysis::analyze_def(def, &mut type_info, namespace).and_then(|rule_index| {
                        def.name
                            .as_ref()
                            .map(|ident| ident.name.clone())
                            .map(|name| module_ast.name.clone().extend(name))
                            .map(|global_ident| (global_ident, rule_index))
                    })
                }
                Global::Let(l) => {
                    analysis::analyze_let(l, &mut type_info, namespace).and_then(|rule_index| {
                        l.name
                            .as_ref()
                            .map(|ident| ident.name.clone())
                            .map(|name| module_ast.name.clone().extend(name))
                            .map(|global_ident| (global_ident, rule_index))
                    })
                }
                Global::Init(init) => {
                    init.analyze(&mut type_info, Context::Expr, namespace);
                    None
                }
                Global::Use(_) | Global::Error(_) => {
                    // no types to check
                    None
                }
            }
        })
        .collect();
    // TODO: add import rules to ident_rules
    validate_idents(&mut type_info, &ident_rules);
    assertions::check(&mut type_info);
    type_info
}

/// Validate all `Ident` rules.
///
/// If the ident is defined, its rule is replaced by a `ValidIdent`-rule,
/// containing the original rule index.
/// Otherwise the rule type is `Unknown`.
fn validate_idents(type_info: &mut TypeInfo, ident_rules: &HashMap<GlobalIdent, RuleIndex>) {
    for rule in type_info.rules.iter_mut() {
        if let TypeDescription::Ident(ident) = &rule.type_description {
            let rule_index = ident_rules.get(ident).copied();
            rule.type_description = rule_index
                .map(TypeDescription::ValidIdent)
                .unwrap_or(TypeDescription::Unknown);
        };
    }
}
