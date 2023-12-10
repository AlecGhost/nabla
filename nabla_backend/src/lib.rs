use std::str::FromStr;

use nabla_frontend::eval::Value;

pub fn to_json_value(value: Value) -> Option<serde_json::Value> {
    match value {
        Value::Unknown | Value::Ref(_) => None,
        Value::Null => Some(serde_json::Value::Null),
        Value::Bool(b) => Some(serde_json::Value::Bool(b)),
        Value::Number(n) => {
            let number = serde_json::Number::from_str(&n).ok()?;
            Some(serde_json::Value::Number(number))
        }
        Value::String(s) => Some(serde_json::Value::String(s)),
        Value::List(list) => {
            let len = list.len();
            let array: Vec<_> = list.into_iter().flat_map(to_json_value).collect();
            if array.len() != len {
                None
            } else {
                Some(serde_json::Value::Array(array))
            }
        }
        Value::Struct(s) => {
            let len = s.len();
            let object: serde_json::Map<_, _> = s
                .into_iter()
                .filter_map(|(k, v)| to_json_value(v).map(|v| (k, v)))
                .collect();
            if object.len() != len {
                None
            } else {
                Some(serde_json::Value::Object(object))
            }
        }
    }
}
