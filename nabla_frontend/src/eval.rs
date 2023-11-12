use crate::ast::{
    Bool, Expr, List, Named, Primitive, PrimitiveValue, Single, Struct, StructOrList,
};
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
        let list = value.map(|v| v.into());
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

pub fn eval(expr: &Expr) -> Value {
    expr.eval()
}

trait Eval {
    fn eval(&self) -> Value;
}

impl Eval for Expr {
    fn eval(&self) -> Value {
        match self {
            Self::Single(single) => single.eval(),
            _ => Value::Unknown,
        }
    }
}

impl Eval for Single {
    fn eval(&self) -> Value {
        match self {
            Self::Struct(s) => s.eval(),
            Self::List(list) => list.eval(),
            Self::Named(named) => named.eval(),
            Self::Primitive(primitive) => primitive.eval(),
        }
    }
}

impl Eval for Named {
    fn eval(&self) -> Value {
        match &self.expr {
            Some(StructOrList::Struct(s)) => s.eval(),
            Some(StructOrList::List(list)) => list.eval(),
            None => Value::Unknown,
        }
    }
}

impl Eval for Struct {
    fn eval(&self) -> Value {
        Value::Struct(
            self.fields
                .iter()
                .flatten()
                .map(|field| {
                    let value = field
                        .expr
                        .as_ref()
                        .map(|expr| expr.eval())
                        .unwrap_or(Value::Unknown);
                    (field.name.name.clone(), value)
                })
                .collect(),
        )
    }
}

impl Eval for List {
    fn eval(&self) -> Value {
        Value::List(self.exprs.iter().map(|expr| expr.eval()).collect())
    }
}

impl Eval for Primitive {
    fn eval(&self) -> Value {
        match self {
            Self::String(PrimitiveValue { value, .. }) => Value::String(value.clone()),
            Self::Char(PrimitiveValue { value, .. }) => Value::String(value.clone()),
            Self::Number(PrimitiveValue { value, .. }) => Value::Number(value.clone()),
            Self::Bool(Bool { value, .. }) => Value::Bool(*value),
            Self::Null(_) => Value::Null,
        }
    }
}
