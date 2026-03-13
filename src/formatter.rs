use crate::model::InputFormat;

// schema 出力などの整形用モジュール。
// 具体実装は後続の単位で追加する。

pub fn format_schema_report_placeholder(format: InputFormat) -> String {
    format!("Format: {:?}\n", format)
}

