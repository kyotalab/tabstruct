pub mod csv;
pub mod json;
pub mod yaml;

use crate::error::TabstructError;
use crate::model::{Document, InputFormat};

/// 指定形式に応じて入力文字列をパースし、`Document` を返す。
/// JSON/YAML はルートが object または array である必要がある。
/// CSV はこの単位では未実装。
pub fn parse_document(format: InputFormat, content: &str) -> Result<Document, TabstructError> {
    match format {
        InputFormat::Json => json::parse_json_document(content),
        InputFormat::Yaml => yaml::parse_yaml_document(content),
        InputFormat::Csv => Err(TabstructError::internal(
            "CSV parsing is not implemented in this unit",
        )),
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
    fn parse_document_csv_unimplemented() {
        let err = parse_document(InputFormat::Csv, "x,y\n1,2").unwrap_err();
        match &err {
            TabstructError::Internal { message } => {
                assert!(message.contains("CSV") || message.contains("not implemented"));
            }
            _ => panic!("expected Internal error"),
        }
    }
}

