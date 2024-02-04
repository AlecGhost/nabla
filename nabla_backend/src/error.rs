use std::num::{ParseFloatError, ParseIntError};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum JsonValueError {
    #[error(transparent)]
    UnknownValueError(#[from] UnknownValueError),
    #[error(transparent)]
    NumberParseError(#[from] serde_json::Error),
}

#[derive(Debug, Error)]
pub enum YamlValueError {
    #[error(transparent)]
    UnknownValueError(#[from] UnknownValueError),
    #[error(transparent)]
    NumberParseError(#[from] serde_yaml::Error),
}

#[derive(Debug, Error)]
pub enum TomlValueError {
    #[error(transparent)]
    UnknownValueError(#[from] UnknownValueError),
    #[error(transparent)]
    ParseIntError(#[from] ParseIntError),
    #[error(transparent)]
    ParseFloatError(#[from] ParseFloatError),
}

#[derive(Debug, Error)]
pub enum XmlValueError {
    #[error(transparent)]
    UnknownValueError(#[from] UnknownValueError),
    #[error("list must be contained inside a struct")]
    StructlessList,
}

#[derive(Clone, Copy, Debug, Error)]
#[error("value is (partially) unknown")]
pub struct UnknownValueError;
