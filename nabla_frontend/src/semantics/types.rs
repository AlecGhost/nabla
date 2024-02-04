use crate::{
    ast::*,
    semantics::{
        error::{Error, ErrorMessage},
        types::analysis::TypeAnalyzer,
    },
    token::ToTokenRange,
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
    Ident(Ident),
    ValidIdent(RuleIndex),
    Primitive(Primitive),
    Rule(RuleIndex),
    Import(GlobalIdent),
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

#[derive(Clone, Copy, Debug)]
enum Context {
    Expr,
    TypeExpr,
}

#[derive(Clone, Debug, Default)]
pub struct TypeInfo<'a> {
    pub rules: Vec<Rule>,
    pub assertions: Vec<(RuleIndex, RuleIndex)>,
    pub idents: HashMap<&'a Ident, RuleIndex>,
    pub errors: Vec<Error>,
}

pub fn analyze(module_ast: &ModuleAst) -> TypeInfo {
    let mut type_info = TypeInfo::default();
    for global in &module_ast.ast.globals {
        match global {
            Global::Use(u) => analysis::analyze_use(u, &mut type_info),
            Global::Def(def) => analysis::analyze_def(def, &mut type_info),
            Global::Let(l) => analysis::analyze_let(l, &mut type_info),
            Global::Init(init) => {
                init.analyze(&mut type_info, Context::Expr);
            }
            Global::Error(_) => { /* no types to check */ }
        }
    }
    validate_idents(&mut type_info);
    lookup_imports(&mut type_info);
    assertions::check(&mut type_info);
    type_info
}

/// Lookup all `Import` rules.
///
/// Imports are looked up and replaced by their rule type, if present,
/// and an `Unknown`-rule otherwise.
/// Currently the lookup is not implemented, so every import is `Unknown`.
fn lookup_imports(type_info: &mut TypeInfo) {
    for rule in type_info.rules.iter_mut() {
        if let TypeDescription::Import(_) = &rule.type_description {
            // currently not implementing use
            rule.type_description = TypeDescription::Unknown;
        };
    }
}

/// Validate all `Ident` rules.
///
/// If the ident is defined, its rule is replaced by a `ValidIdent`-rule,
/// containing the original rule index.
/// If not, and it's a built in, it gets converted to a `BuiltIn`-rule.
/// Otherwise an error is reported and the rule type is `Unknown`.
fn validate_idents(type_info: &mut TypeInfo) {
    for rule in type_info.rules.iter_mut() {
        if let TypeDescription::Ident(ident) = &rule.type_description {
            let rule_index = type_info.idents.get(ident).copied();
            let new_description = if let Some(rule_index) = rule_index {
                TypeDescription::ValidIdent(rule_index)
            } else if let Some(built_in) =
                BuiltInType::into_iter().find(|built_in| built_in.as_str() == ident.name)
            {
                TypeDescription::BuiltIn(built_in)
            } else {
                // TODO: check imports
                type_info.errors.push(Error::new(
                    ErrorMessage::UndefinedIdent(ident.name.clone()),
                    ident.info.to_token_range(),
                ));
                TypeDescription::Unknown
            };
            rule.type_description = new_description;
        };
    }
}
