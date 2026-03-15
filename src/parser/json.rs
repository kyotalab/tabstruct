//! JSON パーサ: 文字列 → Document、serde_json::Value → DataValue。

use crate::error::TabstructError;
use crate::model::{DataValue, Document, InputFormat};
use std::collections::BTreeMap;

/// `serde_json::Value` を内部表現 `DataValue` に変換する。
pub fn value_to_data_value(v: &serde_json::Value) -> DataValue {
    match v {
        serde_json::Value::Null => DataValue::Null,
        serde_json::Value::Bool(b) => DataValue::Bool(*b),
        serde_json::Value::Number(n) => number_to_data_value(n),
        serde_json::Value::String(s) => DataValue::String(s.clone()),
        serde_json::Value::Array(arr) => {
            DataValue::Array(arr.iter().map(value_to_data_value).collect())
        }
        serde_json::Value::Object(obj) => DataValue::Object(
            obj.iter()
                .map(|(k, v)| (k.clone(), value_to_data_value(v)))
                .collect::<BTreeMap<_, _>>(),
        ),
    }
}

fn number_to_data_value(n: &serde_json::Number) -> DataValue {
    if let Some(i) = n.as_i64() {
        return DataValue::Integer(i);
    }
    if let Some(f) = n.as_f64() {
        return DataValue::Float(f);
    }
    // 非常に大きな整数など
    if let Some(u) = n.as_u64() {
        let i = u as i64;
        if i as u64 == u {
            return DataValue::Integer(i);
        }
        return DataValue::Float(u as f64);
    }
    DataValue::Float(n.as_f64().unwrap_or(0.0))
}

/// JSON 文字列をパースし、ルートが object または array の場合に `Document` を返す。
/// ルートがスカラー等の場合はエラーにする。
pub fn parse_json_document(input: &str) -> Result<Document, TabstructError> {
    let value: serde_json::Value =
        serde_json::from_str(input).map_err(|e| TabstructError::JsonParse {
            message: e.to_string(),
        })?;

    let root = value_to_data_value(&value);
    match &root {
        DataValue::Object(_) | DataValue::Array(_) => {}
        _ => {
            return Err(TabstructError::JsonParse {
                message: "Root must be object or array".to_string(),
            });
        }
    }

    Ok(Document {
        format: InputFormat::Json,
        root,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::DataValue;

    #[test]
    fn json_value_null() {
        let v = serde_json::Value::Null;
        assert!(matches!(value_to_data_value(&v), DataValue::Null));
    }

    #[test]
    fn json_value_bool() {
        let v = serde_json::Value::Bool(true);
        assert!(matches!(value_to_data_value(&v), DataValue::Bool(true)));
    }

    #[test]
    fn json_value_integer() {
        let v = serde_json::json!(42);
        assert!(matches!(value_to_data_value(&v), DataValue::Integer(42)));
    }

    #[test]
    fn json_value_float() {
        let v = serde_json::json!(3.14);
        match value_to_data_value(&v) {
            DataValue::Float(f) => assert!((f - 3.14).abs() < 1e-10),
            _ => panic!("expected Float"),
        }
    }

    #[test]
    fn json_value_string() {
        let v = serde_json::Value::String("hello".into());
        match value_to_data_value(&v) {
            DataValue::String(s) => assert_eq!(s, "hello"),
            _ => panic!("expected String"),
        }
    }

    #[test]
    fn json_value_array() {
        let v = serde_json::json!([1, "a", true]);
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
    fn json_value_object() {
        let v = serde_json::json!({ "a": 1, "b": "two" });
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
    fn parse_json_document_object() {
        let input = r#"{"name":"test","count":2}"#;
        let doc = parse_json_document(input).unwrap();
        assert!(matches!(doc.format, InputFormat::Json));
        match &doc.root {
            DataValue::Object(m) => {
                assert_eq!(m.get("name"), Some(&DataValue::String("test".into())));
                assert_eq!(m.get("count"), Some(&DataValue::Integer(2)));
            }
            _ => panic!("expected Object root"),
        }
    }

    #[test]
    fn parse_json_document_array() {
        let input = r#"[{"id":1},{"id":2}]"#;
        let doc = parse_json_document(input).unwrap();
        assert!(matches!(doc.format, InputFormat::Json));
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
    fn parse_json_document_nested() {
        let input = r#"{"settings":{"interval":5,"url":"https://example.com"}}"#;
        let doc = parse_json_document(input).unwrap();
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
    fn parse_json_document_root_not_object_or_array() {
        let err = parse_json_document("42").unwrap_err();
        match &err {
            TabstructError::JsonParse { message } => {
                assert!(message.contains("object or array") || message.contains("Root"));
            }
            _ => panic!("expected JsonParse error"),
        }
    }

    #[test]
    fn parse_json_syntax_error() {
        let err = parse_json_document("{ invalid }").unwrap_err();
        match &err {
            TabstructError::JsonParse { .. } => {}
            _ => panic!("expected JsonParse error"),
        }
    }
}
