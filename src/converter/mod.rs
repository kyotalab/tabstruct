pub mod csv_to_model;
pub mod model_to_csv;
pub mod model_to_json;
pub mod model_to_yaml;

use crate::cli::OutputType;
use crate::model::Document;
use crate::error::TabstructError;

pub fn convert(_doc: &Document, _output: OutputType) -> Result<String, TabstructError> {
    // 変換処理本体は後続単位で実装する
    Err(TabstructError::internal(
        "convert is not implemented yet",
    ))
}

