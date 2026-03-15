pub mod csv_to_model;
pub mod model_to_csv;
pub mod model_to_json;
pub mod model_to_yaml;

use crate::cli::OutputType;
use crate::error::TabstructError;
use crate::model::Document;

/// Document を指定出力形式に変換する。
/// JSON/YAML は Document の root をその形式でシリアライズする（一度 Document に載せたうえで変換）。
pub fn convert(doc: &Document, output: OutputType) -> Result<String, TabstructError> {
    match output {
        OutputType::Json => model_to_json::document_to_json(doc),
        OutputType::Yaml => model_to_yaml::document_to_yaml(doc),
        OutputType::Csv => model_to_csv::document_to_csv(doc),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{DataValue, Document, InputFormat};
    use crate::parser;
    use std::collections::BTreeMap;

    #[test]
    fn convert_json_to_yaml() {
        let json = r#"{"a":1,"b":"x","nested":{"c":true}}"#;
        let doc = parser::parse_document(InputFormat::Json, json).unwrap();
        let yaml = convert(&doc, OutputType::Yaml).unwrap();
        let parsed: serde_yaml::Value = serde_yaml::from_str(&yaml).unwrap();
        let map = parsed.as_mapping().unwrap();
        assert_eq!(
            map.get(&serde_yaml::Value::String("a".into()))
                .and_then(|v| v.as_i64()),
            Some(1)
        );
        assert_eq!(
            map.get(&serde_yaml::Value::String("b".into()))
                .and_then(|v| v.as_str()),
            Some("x")
        );
        let nested = map
            .get(&serde_yaml::Value::String("nested".into()))
            .unwrap()
            .as_mapping()
            .unwrap();
        assert_eq!(
            nested
                .get(&serde_yaml::Value::String("c".into()))
                .and_then(|v| v.as_bool()),
            Some(true)
        );
    }

    #[test]
    fn convert_yaml_to_json() {
        let yaml = "a: 1\nb: x\nnested:\n  c: true";
        let doc = parser::parse_document(InputFormat::Yaml, yaml).unwrap();
        let json = convert(&doc, OutputType::Json).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        let obj = parsed.as_object().unwrap();
        assert_eq!(obj.get("a").and_then(|v| v.as_i64()), Some(1));
        assert_eq!(obj.get("b").and_then(|v| v.as_str()), Some("x"));
        assert!(obj.get("nested").and_then(|v| v.as_object()).is_some());
        assert_eq!(
            obj.get("nested")
                .and_then(|v| v.get("c"))
                .and_then(|v| v.as_bool()),
            Some(true)
        );
    }

    #[test]
    fn convert_json_to_csv() {
        let mut obj = BTreeMap::new();
        obj.insert("id".to_string(), DataValue::Integer(1));
        obj.insert("name".to_string(), DataValue::String("x".to_string()));
        let doc = Document {
            format: InputFormat::Json,
            root: DataValue::Object(obj),
        };
        let csv = convert(&doc, OutputType::Csv).unwrap();
        let lines: Vec<&str> = csv.lines().collect();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "id,name");
        assert_eq!(lines[1], "1,x");
    }
}
