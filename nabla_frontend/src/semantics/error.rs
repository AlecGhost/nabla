use crate::token::TokenRange;
use thiserror::Error;

/// Semantic error
/// Contains an error message and the token range, where the error occurred.
#[derive(Clone, Debug, Error, PartialEq, Eq)]
#[error("{message}")]
pub struct Error {
    pub message: ErrorMessage,
    pub range: TokenRange,
}

impl Error {
    pub const fn new(message: ErrorMessage, range: TokenRange) -> Self {
        Self { message, range }
    }
}

/// Semantic error message
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ErrorMessage {
    AliasMustBeString,
    AliasMustBeIdent,
    AliasingNonSingle,
    MissingField(String),
    MultipleListTypes,
    MultipleInits,
    Redeclaration(String),
    SelfReference(String),
    TypeMismatch,
    UndefinedIdent(String),
    UnexpecedField(String),
    UnexpecedListElement,
    UnionInInit,
    UnknownType,
    Unsupported(String),
    ValueMismatch(String, String),
}

impl std::fmt::Display for ErrorMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::AliasMustBeString => "alias must be a string".to_string(),
            Self::AliasMustBeIdent => "alias must be an identifier".to_string(),
            Self::AliasingNonSingle => "only single use items can be aliased".to_string(),
            Self::MissingField(field_name) => format!("missing field: `{}`", field_name),
            Self::MultipleListTypes => "more than one type in list".to_string(),
            Self::MultipleInits => "more than one initialization".to_string(),
            Self::Redeclaration(ident) => format!("`{}` was alreay declared", ident),
            Self::SelfReference(ident) => format!("`{}` references itself", ident),
            Self::TypeMismatch => "types do not match".to_string(),
            Self::UndefinedIdent(ident) => format!("`{}` is not defined", ident),
            Self::UnexpecedField(field_name) => format!("unexpected field: `{}`", field_name),
            Self::UnexpecedListElement => "unexpected element in list".to_string(),
            Self::UnionInInit => "unions cannot be used in initializations".to_string(),
            Self::UnknownType => "unknown type".to_string(),
            Self::Unsupported(name) => format!("{} is currently unsupported", name),
            Self::ValueMismatch(r#type, value) => {
                format!("`{}` does not match type {}", value, r#type)
            }
        };
        write!(f, "{}", message)
    }
}
