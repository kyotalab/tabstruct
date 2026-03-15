//! 内部モデル (Document) -> JSON への変換。
//! JSON は pretty 出力とする（設計書 10.4）。
//! DataValue をそのまま serde すると enum タグが付くため、
//! 一旦 serde_json::Value に変換してからシリアライズする。

use crate::error::TabstructError;
use crate::model::{DataValue, Document};

/// DataValue を serde_json::Value に変換する（標準の JSON オブジェクト/配列出力のため）。
fn data_value_to_serde_value(v: &DataValue) -> serde_json::Value {
    match v {
        DataValue::Null => serde_json::Value::Null,
        DataValue::Bool(b) => serde_json::Value::Bool(*b),
        DataValue::Integer(i) => serde_json::Value::Number(serde_json::Number::from(*i)),
        DataValue::Float(f) => serde_json::Number::from_f64(*f)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null),
        DataValue::String(s) => serde_json::Value::String(s.clone()),
        DataValue::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(data_value_to_serde_value).collect())
        }
        DataValue::Object(map) => serde_json::Value::Object(
            map.iter()
                .map(|(k, v)| (k.clone(), data_value_to_serde_value(v)))
                .collect::<serde_json::Map<_, _>>(),
        ),
    }
}

/// Document の root を JSON 文字列（pretty）にシリアライズする。
pub fn document_to_json(doc: &Document) -> Result<String, TabstructError> {
    let value = data_value_to_serde_value(&doc.root);
    serde_json::to_string_pretty(&value).map_err(|e| TabstructError::internal(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{DataValue, Document, InputFormat};
    use std::collections::BTreeMap;

    #[test]
    fn document_to_json_object() {
        let mut obj = BTreeMap::new();
        obj.insert("a".to_string(), DataValue::Integer(1));
        obj.insert("b".to_string(), DataValue::String("x".to_string()));
        let doc = Document {
            format: InputFormat::Json,
            root: DataValue::Object(obj),
        };
        let json = document_to_json(&doc).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        let obj = parsed.as_object().unwrap();
        assert_eq!(obj.get("a").and_then(|v| v.as_i64()), Some(1));
        assert_eq!(obj.get("b").and_then(|v| v.as_str()), Some("x"));
    }

    #[test]
    fn document_to_json_array() {
        let doc = Document {
            format: InputFormat::Json,
            root: DataValue::Array(vec![
                DataValue::Bool(true),
                DataValue::Null,
                DataValue::String("s".to_string()),
            ]),
        };
        let json = document_to_json(&doc).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        let arr = parsed.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0].as_bool(), Some(true));
        assert!(arr[1].is_null());
        assert_eq!(arr[2].as_str(), Some("s"));
    }

    #[test]
    fn document_to_json_pretty_has_newlines() {
        let mut obj = BTreeMap::new();
        obj.insert("k".to_string(), DataValue::Integer(1));
        let doc = Document {
            format: InputFormat::Json,
            root: DataValue::Object(obj),
        };
        let json = document_to_json(&doc).unwrap();
        assert!(
            json.contains('\n'),
            "JSON output should be pretty (contain newlines)"
        );
    }
}
