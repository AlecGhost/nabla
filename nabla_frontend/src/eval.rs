use crate::ast::{
    Bool, Expr, List, Named, Primitive, PrimitiveValue, Single, Struct, StructOrList,
};
pub use value::Value;

mod value;

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
                        .map(Expr::eval)
                        .unwrap_or(Value::Unknown);
                    (field.emit_name().to_string(), value)
                })
                .collect(),
        )
    }
}

impl Eval for List {
    fn eval(&self) -> Value {
        Value::List(self.exprs.iter().map(Expr::eval).collect())
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
