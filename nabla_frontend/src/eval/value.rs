use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    Unknown,
    Null,
    Bool(bool),
    Number(String),
    String(String),
    List(Vec<Value>),
    Struct(HashMap<String, Value>),
}

impl Value {
    /// Returns whether this value is entirely known/does not contain `Value::Unknown`.
    pub fn is_known(&self) -> bool {
        match self {
            Self::Unknown => false,
            Self::Null | Self::Bool(_) | Self::Number(_) | Self::String(_) => true,
            Self::List(l) => l.iter().all(Self::is_known),
            Self::Struct(s) => s.values().all(Self::is_known),
        }
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<usize> for Value {
    fn from(value: usize) -> Self {
        Self::Number(value.to_string())
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Self::Number(value.to_string())
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Self::from(value.to_string())
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl<V, const N: usize> From<[V; N]> for Value
where
    V: Into<Self>,
{
    fn from(value: [V; N]) -> Self {
        let list = value.map(V::into);
        Self::List(Vec::from(list))
    }
}

impl<K, V, const N: usize> From<[(K, V); N]> for Value
where
    K: Into<String>,
    V: Into<Self>,
{
    fn from(value: [(K, V); N]) -> Self {
        let map: [(String, Self); N] = value.map(|(k, v)| (k.into(), v.into()));
        Self::Struct(HashMap::from(map))
    }
}
