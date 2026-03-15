//! 内部モデル (Document) -> YAML への変換。
//! YAML は標準的な可読形式とする（設計書 10.4）。
//! DataValue をそのまま serde_yaml すると型タグ（!Object 等）が出るため、
//! 一旦 serde_json::Value に変換してから YAML 化する。

use crate::error::TabstructError;
use crate::model::{DataValue, Document};

/// DataValue を serde_json::Value に変換する（YAML 用の標準スカラー出力のため）。
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

/// Document の root を YAML 文字列にシリアライズする（標準的な可読形式）。
pub fn document_to_yaml(doc: &Document) -> Result<String, TabstructError> {
    let value = data_value_to_serde_value(&doc.root);
    serde_yaml::to_string(&value).map_err(|e| TabstructError::internal(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{DataValue, Document, InputFormat};
    use std::collections::BTreeMap;

    #[test]
    fn document_to_yaml_object() {
        let mut obj = BTreeMap::new();
        obj.insert("a".to_string(), DataValue::Integer(1));
        obj.insert("b".to_string(), DataValue::String("x".to_string()));
        let doc = Document {
            format: InputFormat::Yaml,
            root: DataValue::Object(obj),
        };
        let yaml = document_to_yaml(&doc).unwrap();
        let parsed: serde_yaml::Value = serde_yaml::from_str(&yaml).unwrap();
        let map = parsed.as_mapping().unwrap();
        assert_eq!(map.get(&serde_yaml::Value::String("a".into())).and_then(|v| v.as_i64()), Some(1));
        assert_eq!(map.get(&serde_yaml::Value::String("b".into())).and_then(|v| v.as_str()), Some("x"));
    }

    #[test]
    fn document_to_yaml_array() {
        let doc = Document {
            format: InputFormat::Yaml,
            root: DataValue::Array(vec![
                DataValue::Bool(true),
                DataValue::Null,
                DataValue::String("s".to_string()),
            ]),
        };
        let yaml = document_to_yaml(&doc).unwrap();
        let parsed: serde_yaml::Value = serde_yaml::from_str(&yaml).unwrap();
        let seq = parsed.as_sequence().unwrap();
        assert_eq!(seq.len(), 3);
        assert_eq!(seq[0].as_bool(), Some(true));
        assert!(seq[1].is_null() || seq[1].as_str() == Some("~"));
        assert_eq!(seq[2].as_str(), Some("s"));
    }

    #[test]
    fn document_to_yaml_nested() {
        let mut inner = BTreeMap::new();
        inner.insert("n".to_string(), DataValue::Integer(2));
        let mut outer = BTreeMap::new();
        outer.insert("x".to_string(), DataValue::Object(inner));
        let doc = Document {
            format: InputFormat::Yaml,
            root: DataValue::Object(outer),
        };
        let yaml = document_to_yaml(&doc).unwrap();
        let parsed: serde_yaml::Value = serde_yaml::from_str(&yaml).unwrap();
        let map = parsed.as_mapping().unwrap();
        let x = map.get(&serde_yaml::Value::String("x".into())).unwrap();
        let inner_map = x.as_mapping().unwrap();
        assert_eq!(inner_map.get(&serde_yaml::Value::String("n".into())).and_then(|v| v.as_i64()), Some(2));
    }
}
