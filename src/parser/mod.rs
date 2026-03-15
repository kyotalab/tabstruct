pub mod csv;
pub mod json;
pub mod yaml;

use crate::error::TabstructError;
use crate::model::{Document, InputFormat};

/// 指定形式に応じて入力文字列をパースし、`Document` を返す。
/// JSON/YAML はルートが object または array である必要がある。
/// CSV は RawCsvTable → TypedCsvTable → Document で変換する。
pub fn parse_document(format: InputFormat, content: &str) -> Result<Document, TabstructError> {
    match format {
        InputFormat::Json => json::parse_json_document(content),
        InputFormat::Yaml => yaml::parse_yaml_document(content),
        InputFormat::Csv => {
            let raw = csv::parse_csv(content)?;
            let typed = csv::raw_to_typed(raw)?;
            Ok(csv::typed_table_to_document(typed))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::DataValue;

    #[test]
    fn parse_document_json_branch() {
        let doc = parse_document(InputFormat::Json, r#"{"a":1}"#).unwrap();
        assert!(matches!(doc.format, InputFormat::Json));
        assert!(matches!(&doc.root, DataValue::Object(_)));
    }

    #[test]
    fn parse_document_yaml_branch() {
        let doc = parse_document(InputFormat::Yaml, "a: 1").unwrap();
        assert!(matches!(doc.format, InputFormat::Yaml));
        assert!(matches!(&doc.root, DataValue::Object(_)));
    }

    #[test]
    fn parse_document_csv_parses_to_array_of_objects() {
        let doc = parse_document(InputFormat::Csv, "id,name\n1,alice\n2,bob").unwrap();
        assert!(matches!(doc.format, InputFormat::Csv));
        let arr = match &doc.root {
            DataValue::Array(a) => a,
            _ => panic!("expected array root"),
        };
        assert_eq!(arr.len(), 2);
        let first = match &arr[0] {
            DataValue::Object(m) => m,
            _ => panic!("expected object"),
        };
        assert_eq!(first.get("id"), Some(&DataValue::Integer(1)));
        assert_eq!(first.get("name"), Some(&DataValue::String("alice".into())));
    }
}

