pub mod csv;
pub mod json;
pub mod yaml;

use crate::model::{Document, InputFormat};
use crate::error::TabstructError;

pub fn parse_document(_format: InputFormat, _content: &str) -> Result<Document, TabstructError> {
    // 実際のパース処理は後続単位で実装する
    Err(TabstructError::internal(
        "parse_document is not implemented yet",
    ))
}

