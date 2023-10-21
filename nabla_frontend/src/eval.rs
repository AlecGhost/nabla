use crate::ast::{
    Bool, Expr, List, Named, Primitive, PrimitiveValue, Single, Struct, StructOrList,
};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    Unknown,
    None,
    Bool(bool),
    Number(String),
    String(String),
    List(Vec<Value>),
    Struct(HashMap<String, Value>),
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
            Expr::Single(single) => single.eval(),
            _ => Value::Unknown,
        }
    }
}

impl Eval for Single {
    fn eval(&self) -> Value {
        match self {
            Single::Struct(s) => s.eval(),
            Single::List(list) => list.eval(),
            Single::Named(named) => named.eval(),
            Single::Primitive(primitive) => primitive.eval(),
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
            Primitive::String(PrimitiveValue { value, .. }) => Value::String(value.clone()),
            Primitive::Char(PrimitiveValue { value, .. }) => Value::String(value.clone()),
            Primitive::Number(PrimitiveValue { value, .. }) => Value::Number(value.clone()),
            Primitive::Bool(Bool { value, .. }) => Value::Bool(*value),
        }
    }
}
