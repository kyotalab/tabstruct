pub mod csv;
pub mod json;
pub mod yaml;

use crate::error::TabstructError;
use crate::model::{Document, InputFormat};

/// 指定形式に応じて入力文字列をパースし、`Document` を返す。
/// JSON/YAML のみ対応。CSV は app 層で parser::csv + converter::csv_to_model を用いて処理する。
pub fn parse_document(format: InputFormat, content: &str) -> Result<Document, TabstructError> {
    match format {
        InputFormat::Json => json::parse_json_document(content),
        InputFormat::Yaml => yaml::parse_yaml_document(content),
        InputFormat::Csv => Err(TabstructError::internal(
            "parse_document does not handle CSV; use parser::csv + converter::csv_to_model",
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

    // CSV → Document は app 層で parser::csv + converter::csv_to_model により行う。
    // 統合テストは converter::csv_to_model::tests および app の get_document で実施。
}

