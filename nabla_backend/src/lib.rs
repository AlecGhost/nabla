use error::{JsonValueError, TomlValueError, UnknownValueError, XmlValueError, YamlValueError};
use nabla_frontend::eval::Value;
use std::str::FromStr;
use xml_builder::XMLElement;

pub mod error;

pub fn to_json_value(value: Value) -> Result<serde_json::Value, JsonValueError> {
    match value {
        Value::Unknown => Err(UnknownValueError)?,
        Value::Null => Ok(serde_json::Value::Null),
        Value::Bool(b) => Ok(serde_json::Value::Bool(b)),
        Value::Number(n) => {
            let number = serde_json::Number::from_str(&n)?;
            Ok(serde_json::Value::Number(number))
        }
        Value::String(s) => Ok(serde_json::Value::String(s)),
        Value::List(list) => {
            let array = list
                .into_iter()
                .map(to_json_value)
                .collect::<Result<Vec<_>, JsonValueError>>()?;
            Ok(serde_json::Value::Array(array))
        }
        Value::Struct(s) => {
            let object = s
                .into_iter()
                .map(|(k, v)| to_json_value(v).map(|v| (k, v)))
                .collect::<Result<serde_json::Map<_, _>, JsonValueError>>()?;
            Ok(serde_json::Value::Object(object))
        }
    }
}

pub fn to_yaml_value(value: Value) -> Result<serde_yaml::Value, YamlValueError> {
    match value {
        Value::Unknown => Err(UnknownValueError)?,
        Value::Null => Ok(serde_yaml::Value::Null),
        Value::Bool(b) => Ok(serde_yaml::Value::Bool(b)),
        Value::Number(n) => {
            let number = serde_yaml::Number::from_str(&n)?;
            Ok(serde_yaml::Value::Number(number))
        }
        Value::String(s) => Ok(serde_yaml::Value::String(s)),
        Value::List(list) => {
            let array = list
                .into_iter()
                .map(to_yaml_value)
                .collect::<Result<Vec<_>, YamlValueError>>()?;
            Ok(serde_yaml::Value::Sequence(array))
        }
        Value::Struct(s) => {
            let object = s
                .into_iter()
                .map(|(k, v)| to_yaml_value(v).map(|v| (serde_yaml::Value::String(k), v)))
                .collect::<Result<serde_yaml::Mapping, YamlValueError>>()?;
            Ok(serde_yaml::Value::Mapping(object))
        }
    }
}

pub fn to_toml_value(value: Value) -> Result<Option<toml::Value>, TomlValueError> {
    match value {
        Value::Unknown => Err(UnknownValueError)?,
        Value::Null => Ok(None),
        Value::Bool(b) => Ok(Some(toml::Value::Boolean(b))),
        Value::Number(n) => {
            if n.contains('.') {
                let float = f64::from_str(&n)?;
                Ok(Some(toml::Value::Float(float)))
            } else {
                let int = i64::from_str(&n)?;
                Ok(Some(toml::Value::Integer(int)))
            }
        }
        Value::String(s) => Ok(Some(toml::Value::String(s))),
        Value::List(list) => {
            let array = list
                .into_iter()
                .map(to_toml_value)
                .filter_map(|r| match r {
                    Ok(Some(v)) => Some(Ok(v)),
                    Ok(None) => None,
                    Err(err) => Some(Err(err)),
                })
                .collect::<Result<Vec<_>, TomlValueError>>()?;
            Ok(Some(toml::Value::Array(array)))
        }
        Value::Struct(s) => {
            let object: toml::map::Map<_, _> = s
                .into_iter()
                .map(|(k, v)| to_toml_value(v).map(|v| (k, v)))
                .filter_map(|r| match r {
                    Ok((k, Some(v))) => Some(Ok((k, v))),
                    Ok((_, None)) => None,
                    Err(err) => Some(Err(err)),
                })
                .collect::<Result<toml::map::Map<_, _>, TomlValueError>>()?;
            Ok(Some(toml::Value::Table(object)))
        }
    }
}

pub fn to_xml_value(value: Value, name: &str) -> Result<XMLElement, XmlValueError> {
    let mut element = XMLElement::new(name);
    match value {
        Value::Unknown => Err(UnknownValueError)?,
        Value::Null => {}
        Value::Bool(b) => element.add_text(b.to_string()).expect("Element is empty"),
        Value::Number(n) => element.add_text(n).expect("Element is empty"),
        Value::String(s) => element.add_text(s).expect("Element is empty"),
        Value::List(_) => Err(XmlValueError::StructlessList)?,
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
    Ok(element)
}
