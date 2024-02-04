use nabla_frontend::eval::Value;
use std::str::FromStr;
use xml_builder::XMLElement;

pub fn to_json_value(value: Value) -> Option<serde_json::Value> {
    match value {
        Value::Unknown => None,
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

pub fn to_yaml_value(value: Value) -> Option<serde_yaml::Value> {
    match value {
        Value::Unknown => None,
        Value::Null => Some(serde_yaml::Value::Null),
        Value::Bool(b) => Some(serde_yaml::Value::Bool(b)),
        Value::Number(n) => {
            let number = serde_yaml::Number::from_str(&n).ok()?;
            Some(serde_yaml::Value::Number(number))
        }
        Value::String(s) => Some(serde_yaml::Value::String(s)),
        Value::List(list) => {
            let len = list.len();
            let array: Vec<_> = list.into_iter().flat_map(to_yaml_value).collect();
            if array.len() != len {
                None
            } else {
                Some(serde_yaml::Value::Sequence(array))
            }
        }
        Value::Struct(s) => {
            let len = s.len();
            let object: serde_yaml::Mapping = s
                .into_iter()
                .filter_map(|(k, v)| to_yaml_value(v).map(|v| (serde_yaml::Value::String(k), v)))
                .collect();
            if object.len() != len {
                None
            } else {
                Some(serde_yaml::Value::Mapping(object))
            }
        }
    }
}

pub fn to_toml_value(value: Value) -> Option<toml::Value> {
    match value {
        Value::Unknown => None,
        Value::Null => None,
        Value::Bool(b) => Some(toml::Value::Boolean(b)),
        Value::Number(n) => {
            if n.contains('.') {
                let float = f64::from_str(&n).ok()?;
                Some(toml::Value::Float(float))
            } else {
                let int = i64::from_str(&n).ok()?;
                Some(toml::Value::Integer(int))
            }
        }
        Value::String(s) => Some(toml::Value::String(s)),
        Value::List(list) => {
            let len = list.iter().filter(|v| !matches!(v, Value::Null)).count();
            let array: Vec<_> = list.into_iter().flat_map(to_toml_value).collect();
            if array.len() != len {
                None
            } else {
                Some(toml::Value::Array(array))
            }
        }
        Value::Struct(s) => {
            let len = s.iter().filter(|(_, v)| !matches!(v, Value::Null)).count();
            let object: toml::map::Map<_, _> = s
                .into_iter()
                .filter_map(|(k, v)| to_toml_value(v).map(|v| (k, v)))
                .collect();
            if object.len() != len {
                None
            } else {
                Some(toml::Value::Table(object))
            }
        }
    }
}

pub fn to_xml_value(value: Value, name: &str) -> Option<XMLElement> {
    let mut element = XMLElement::new(name);
    match value {
        Value::Unknown => return None,
        Value::Null => {}
        Value::Bool(b) => element.add_text(b.to_string()).expect("Element is empty"),
        Value::Number(n) => element.add_text(n).expect("Element is empty"),
        Value::String(s) => element.add_text(s).expect("Element is empty"),
        Value::List(_) => return None,
        Value::Struct(s) => {
            for (key, value) in s {
                if let Value::List(list) = value {
                    for value in list {
                        element
                            .add_child(to_xml_value(value, &key)?)
                            .expect("Element is no text element");
                    }
                } else {
                    element
                        .add_child(to_xml_value(value, &key)?)
                        .expect("Element is no text element");
                }
            }
        }
    };
    Some(element)
}
