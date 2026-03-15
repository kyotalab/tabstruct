//! YAML パーサ: 文字列 → Document、serde_yaml::Value → DataValue。

use crate::error::TabstructError;
use crate::model::{DataValue, Document, InputFormat};
use std::collections::BTreeMap;

/// `serde_yaml::Value` を内部表現 `DataValue` に変換する。
/// - Tagged はタグを無視し、内側の value のみを変換する。
/// - Mapping のキーは文字列として扱う（非文字列キーは文字列化する）。
pub fn value_to_data_value(v: &serde_yaml::Value) -> DataValue {
    match v {
        serde_yaml::Value::Null => DataValue::Null,
        serde_yaml::Value::Bool(b) => DataValue::Bool(*b),
        serde_yaml::Value::Number(n) => yaml_number_to_data_value(n),
        serde_yaml::Value::String(s) => DataValue::String(s.clone()),
        serde_yaml::Value::Sequence(seq) => {
            DataValue::Array(seq.iter().map(value_to_data_value).collect())
        }
        serde_yaml::Value::Mapping(map) => {
            let mut obj = BTreeMap::new();
            for (k, val) in map {
                let key_str = yaml_value_to_key_string(k);
                obj.insert(key_str, value_to_data_value(val));
            }
            DataValue::Object(obj)
        }
        serde_yaml::Value::Tagged(tagged) => value_to_data_value(&tagged.value),
    }
}

fn yaml_number_to_data_value(n: &serde_yaml::Number) -> DataValue {
    if let Some(i) = n.as_i64() {
        return DataValue::Integer(i);
    }
    if let Some(f) = n.as_f64() {
        return DataValue::Float(f);
    }
    if let Some(u) = n.as_u64() {
        let i = u as i64;
        if i as u64 == u {
            return DataValue::Integer(i);
        }
        return DataValue::Float(u as f64);
    }
    DataValue::Float(n.as_f64().unwrap_or(0.0))
}

/// Mapping のキーを文字列に変換する。文字列キーはそのまま、それ以外はデバッグ表現を用いる。
fn yaml_value_to_key_string(k: &serde_yaml::Value) -> String {
    if let Some(s) = k.as_str() {
        return s.to_string();
    }
    // 非文字列キー（YAML では数値キー等が可能）は文字列化
    serde_yaml::to_string(k).unwrap_or_else(|_| format!("{:?}", k))
}

/// YAML 文字列をパースし、ルートが object または array の場合に `Document` を返す。
/// ルートがスカラー等の場合はエラーにする。
pub fn parse_yaml_document(input: &str) -> Result<Document, TabstructError> {
    let value: serde_yaml::Value =
        serde_yaml::from_str(input).map_err(|e| TabstructError::YamlParse {
            message: e.to_string(),
        })?;

    let root = value_to_data_value(&value);
    match &root {
        DataValue::Object(_) | DataValue::Array(_) => {}
        _ => {
            return Err(TabstructError::YamlParse {
                message: "Root must be object or array".to_string(),
            });
        }
    }

    Ok(Document {
        format: InputFormat::Yaml,
        root,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::DataValue;

    #[test]
    fn yaml_value_null() {
        let v = serde_yaml::Value::Null;
        assert!(matches!(value_to_data_value(&v), DataValue::Null));
    }

    #[test]
    fn yaml_value_bool() {
        let v = serde_yaml::Value::Bool(true);
        assert!(matches!(value_to_data_value(&v), DataValue::Bool(true)));
    }

    #[test]
    fn yaml_value_integer() {
        let v = serde_yaml::from_str("42").unwrap();
        assert!(matches!(value_to_data_value(&v), DataValue::Integer(42)));
    }

    #[test]
    fn yaml_value_float() {
        let v = serde_yaml::from_str("3.14").unwrap();
        match value_to_data_value(&v) {
            DataValue::Float(f) => assert!((f - 3.14).abs() < 1e-10),
            _ => panic!("expected Float"),
        }
    }

    #[test]
    fn yaml_value_string() {
        let v = serde_yaml::Value::String("hello".into());
        match value_to_data_value(&v) {
            DataValue::String(s) => assert_eq!(s, "hello"),
            _ => panic!("expected String"),
        }
    }

    #[test]
    fn yaml_value_sequence() {
        let v = serde_yaml::from_str("[1, a, true]").unwrap();
        match value_to_data_value(&v) {
            DataValue::Array(arr) => {
                assert_eq!(arr.len(), 3);
                assert!(matches!(&arr[0], DataValue::Integer(1)));
                assert!(matches!(&arr[1], DataValue::String(s) if s == "a"));
                assert!(matches!(&arr[2], DataValue::Bool(true)));
            }
            _ => panic!("expected Array"),
        }
    }

    #[test]
    fn yaml_value_mapping() {
        let v = serde_yaml::from_str("a: 1\nb: two").unwrap();
        match value_to_data_value(&v) {
            DataValue::Object(map) => {
                assert_eq!(map.len(), 2);
                assert!(matches!(map.get("a"), Some(DataValue::Integer(1))));
                assert!(matches!(map.get("b"), Some(DataValue::String(s)) if s == "two"));
            }
            _ => panic!("expected Object"),
        }
    }

    #[test]
    fn parse_yaml_document_object() {
        let input = "name: test\ncount: 2";
        let doc = parse_yaml_document(input).unwrap();
        assert!(matches!(doc.format, InputFormat::Yaml));
        match &doc.root {
            DataValue::Object(m) => {
                assert_eq!(m.get("name"), Some(&DataValue::String("test".into())));
                assert_eq!(m.get("count"), Some(&DataValue::Integer(2)));
            }
            _ => panic!("expected Object root"),
        }
    }

    #[test]
    fn parse_yaml_document_array() {
        let input = "- id: 1\n- id: 2";
        let doc = parse_yaml_document(input).unwrap();
        assert!(matches!(doc.format, InputFormat::Yaml));
        match &doc.root {
            DataValue::Array(arr) => {
                assert_eq!(arr.len(), 2);
                assert!(
                    matches!(&arr[0], DataValue::Object(m) if m.get("id") == Some(&DataValue::Integer(1)))
                );
                assert!(
                    matches!(&arr[1], DataValue::Object(m) if m.get("id") == Some(&DataValue::Integer(2)))
                );
            }
            _ => panic!("expected Array root"),
        }
    }

    #[test]
    fn parse_yaml_document_nested() {
        let input = r#"
settings:
  interval: 5
  url: https://example.com
"#;
        let doc = parse_yaml_document(input.trim()).unwrap();
        match &doc.root {
            DataValue::Object(outer) => {
                let settings = outer.get("settings").unwrap();
                match settings {
                    DataValue::Object(inner) => {
                        assert_eq!(inner.get("interval"), Some(&DataValue::Integer(5)));
                        assert_eq!(
                            inner.get("url"),
                            Some(&DataValue::String("https://example.com".into()))
                        );
                    }
                    _ => panic!("expected nested Object"),
                }
            }
            _ => panic!("expected Object root"),
        }
    }

    #[test]
    fn parse_yaml_document_root_not_object_or_array() {
        let err = parse_yaml_document("42").unwrap_err();
        match &err {
            TabstructError::YamlParse { message } => {
                assert!(message.contains("object or array") || message.contains("Root"));
            }
            _ => panic!("expected YamlParse error"),
        }
    }

    #[test]
    fn parse_yaml_syntax_error() {
        let err = parse_yaml_document("invalid: [unclosed").unwrap_err();
        match &err {
            TabstructError::YamlParse { .. } => {}
            _ => panic!("expected YamlParse error"),
        }
    }
}
