use crate::error::TabstructError;
use crate::model::Document;
use crate::schema::SchemaReport;

pub fn analyze(_doc: &Document) -> Result<SchemaReport, TabstructError> {
    // schema 解析は後続単位で実装する
    Err(TabstructError::internal(
        "schema::analyze is not implemented yet",
    ))
}

