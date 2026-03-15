//! CSV パーサ: RawTable 読込・ヘッダ検証・型推定・TypedCsvTable 変換。

use crate::error::TabstructError;
use crate::model::DataValue;
use crate::schema::types::{DisplayType, PrimitiveKind};
use std::io::Cursor;

/// 生のCSVテーブル（ヘッダ + 行すべて文字列）。
#[derive(Debug, Clone)]
pub struct RawCsvTable {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

/// 列の型候補（型推定用）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnType {
    Boolean,
    Integer,
    Float,
    String,
}

/// 型付きCSVテーブル（ヘッダ・列型・DataValue の行）。
#[derive(Debug, Clone)]
pub struct TypedCsvTable {
    pub headers: Vec<String>,
    pub column_types: Vec<DisplayType>,
    pub rows: Vec<Vec<DataValue>>,
}

/// CSV文字列をパースして RawCsvTable を返す。ヘッダ必須・カンマ区切り。
/// 列数不一致があればエラーにする。
pub fn parse_csv(content: &str) -> Result<RawCsvTable, TabstructError> {
    if content.trim().is_empty() {
        return Err(TabstructError::CsvParse {
            row: 1,
            message: "empty input or missing header".to_string(),
        });
    }
    let mut rdr = csv::Reader::from_reader(Cursor::new(content));
    let headers: Vec<String> = rdr
        .headers()
        .map_err(|e| TabstructError::CsvParse {
            row: 1,
            message: e.to_string(),
        })?
        .iter()
        .map(|s| s.to_string())
        .collect();

    if headers.is_empty() {
        return Err(TabstructError::InvalidCsvHeader {
            column: 1,
            header: String::new(),
        });
    }

    let expected = headers.len();
    let mut rows = Vec::new();
    for (i, result) in rdr.records().enumerate() {
        let row: Vec<String> = result
            .map_err(|e| TabstructError::CsvParse {
                row: i + 2,
                message: e.to_string(),
            })?
            .iter()
            .map(|s| s.to_string())
            .collect();
        if row.len() != expected {
            return Err(TabstructError::CsvColumnCountMismatch {
                row: i + 2,
                expected,
                actual: row.len(),
            });
        }
        rows.push(row);
    }

    Ok(RawCsvTable { headers, rows })
}

/// ヘッダを検証する（空・`.` 開始/終端・`..`・重複）。
pub fn validate_headers(raw: &RawCsvTable) -> Result<(), TabstructError> {
    let mut seen = std::collections::HashSet::new();
    for (col, h) in raw.headers.iter().enumerate() {
        let col_1based = col + 1;
        if h.is_empty() {
            return Err(TabstructError::InvalidCsvHeader {
                column: col_1based,
                header: h.clone(),
            });
        }
        if h.starts_with('.') {
            return Err(TabstructError::InvalidCsvHeader {
                column: col_1based,
                header: h.clone(),
            });
        }
        if h.ends_with('.') {
            return Err(TabstructError::InvalidCsvHeader {
                column: col_1based,
                header: h.clone(),
            });
        }
        if h.contains("..") {
            return Err(TabstructError::InvalidCsvHeader {
                column: col_1based,
                header: h.clone(),
            });
        }
        if !seen.insert(h.as_str()) {
            return Err(TabstructError::DuplicateCsvHeader {
                column: col_1based,
                header: h.clone(),
            });
        }
    }
    Ok(())
}

/// path conflict を検証する（例: `settings` と `settings.interval` は競合）。
pub fn validate_path_conflicts(headers: &[String]) -> Result<(), TabstructError> {
    for a in headers {
        for b in headers {
            if a == b {
                continue;
            }
            let a_dot = format!("{a}.");
            if b.starts_with(&a_dot) {
                return Err(TabstructError::PathConflict { path: a.clone() });
            }
        }
    }
    Ok(())
}

/// セル文字列から型候補を推定する。空なら None（null 候補）。
pub fn infer_cell_type(value: &str) -> Option<ColumnType> {
    if value.is_empty() {
        return None;
    }
    if value == "true" || value == "false" {
        return Some(ColumnType::Boolean);
    }
    if value.parse::<i64>().is_ok() {
        return Some(ColumnType::Integer);
    }
    if value.parse::<f64>().is_ok() {
        return Some(ColumnType::Float);
    }
    Some(ColumnType::String)
}

/// 列型をマージする（integer + float -> float、その他混在 -> string）。
pub fn merge_column_type(
    current: Option<ColumnType>,
    next: Option<ColumnType>,
) -> Option<ColumnType> {
    match (current, next) {
        (None, x) => x,
        (x, None) => x,
        (Some(ColumnType::Boolean), Some(ColumnType::Boolean)) => Some(ColumnType::Boolean),
        (Some(ColumnType::Integer), Some(ColumnType::Integer)) => Some(ColumnType::Integer),
        (Some(ColumnType::Integer), Some(ColumnType::Float)) => Some(ColumnType::Float),
        (Some(ColumnType::Float), Some(ColumnType::Integer)) => Some(ColumnType::Float),
        (Some(ColumnType::Float), Some(ColumnType::Float)) => Some(ColumnType::Float),
        _ => Some(ColumnType::String),
    }
}

/// 列が nullable かどうか（空セル・欠損が1つでもある、またはデータ行が0件のとき true）。
pub fn column_has_nullable(column_index: usize, raw: &RawCsvTable) -> bool {
    if raw.rows.is_empty() {
        return true; // 全空列は string? 扱い
    }
    raw.rows
        .iter()
        .any(|row| row.get(column_index).is_none_or(|s| s.is_empty()))
}

/// 列型 + nullable を DisplayType に変換する。
fn column_type_to_display(ty: ColumnType, nullable: bool) -> DisplayType {
    let kind = match ty {
        ColumnType::Boolean => PrimitiveKind::Boolean,
        ColumnType::Integer => PrimitiveKind::Integer,
        ColumnType::Float => PrimitiveKind::Float,
        ColumnType::String => PrimitiveKind::String,
    };
    DisplayType { kind, nullable }
}

/// セル文字列を列型に従って DataValue に変換する。空なら Null。
pub fn cast_cell(value: &str, ty: ColumnType) -> DataValue {
    if value.is_empty() {
        return DataValue::Null;
    }
    match ty {
        ColumnType::Boolean => DataValue::Bool(value == "true"),
        ColumnType::Integer => DataValue::Integer(value.parse().unwrap_or(0)),
        ColumnType::Float => DataValue::Float(value.parse().unwrap_or(0.0)),
        ColumnType::String => DataValue::String(value.to_string()),
    }
}

/// RawCsvTable を TypedCsvTable に変換する（ヘッダ検証・path conflict 検証を含む）。
pub fn raw_to_typed(raw: RawCsvTable) -> Result<TypedCsvTable, TabstructError> {
    validate_headers(&raw)?;
    validate_path_conflicts(&raw.headers)?;

    let col_count = raw.headers.len();
    let mut column_types: Vec<ColumnType> = Vec::with_capacity(col_count);
    let mut nullables: Vec<bool> = Vec::with_capacity(col_count);

    for col in 0..col_count {
        let mut merged: Option<ColumnType> = None;
        for row in &raw.rows {
            let cell = row.get(col).map(String::as_str).unwrap_or("");
            merged = merge_column_type(merged, infer_cell_type(cell));
        }
        // 非空セルが1つもない列は string? 扱い
        let ty = merged.unwrap_or(ColumnType::String);
        let nullable = column_has_nullable(col, &raw);
        column_types.push(ty);
        nullables.push(nullable);
    }

    let display_types: Vec<DisplayType> = column_types
        .iter()
        .zip(nullables.iter())
        .map(|(&ty, &nullable)| column_type_to_display(ty, nullable))
        .collect();

    let rows: Vec<Vec<DataValue>> = raw
        .rows
        .iter()
        .map(|row| {
            row.iter()
                .zip(column_types.iter())
                .map(|(s, &ty)| cast_cell(s, ty))
                .collect()
        })
        .collect();

    Ok(TypedCsvTable {
        headers: raw.headers,
        column_types: display_types,
        rows,
    })
}

// TypedCsvTable → Document の unflatten は converter::csv_to_model に委譲する。

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::DataValue;
    use crate::schema::types::PrimitiveKind;

    #[test]
    fn parse_normal_csv() {
        let raw = parse_csv("a,b,c\n1,2,3\n4,5,6").unwrap();
        assert_eq!(raw.headers, ["a", "b", "c"]);
        assert_eq!(raw.rows.len(), 2);
        assert_eq!(raw.rows[0], ["1", "2", "3"]);
        assert_eq!(raw.rows[1], ["4", "5", "6"]);
    }

    #[test]
    fn parse_empty_csv_fails_no_header() {
        let err = parse_csv("").unwrap_err();
        assert!(matches!(err, TabstructError::CsvParse { .. }));
    }

    #[test]
    fn parse_csv_column_mismatch() {
        // ヘッダとデータで列数が違うとエラー（csv crate は CsvParse、自前チェックなら CsvColumnCountMismatch）
        let err = parse_csv("a,b,c\n1,2").unwrap_err();
        match &err {
            TabstructError::CsvColumnCountMismatch {
                row,
                expected,
                actual,
            } => {
                assert_eq!(*row, 2);
                assert_eq!(*expected, 3);
                assert_eq!(*actual, 2);
            }
            TabstructError::CsvParse { row, message } => {
                assert_eq!(*row, 2);
                assert!(message.contains("field") || message.contains("record"));
            }
            _ => panic!("expected column count error, got {:?}", err),
        }
    }

    #[test]
    fn validate_headers_empty() {
        let raw = RawCsvTable {
            headers: vec!["a".into(), "".into(), "c".into()],
            rows: vec![],
        };
        let err = validate_headers(&raw).unwrap_err();
        assert!(matches!(
            err,
            TabstructError::InvalidCsvHeader { column: 2, .. }
        ));
    }

    #[test]
    fn validate_headers_starts_with_dot() {
        let raw = RawCsvTable {
            headers: vec![".a".into(), "b".into()],
            rows: vec![],
        };
        let err = validate_headers(&raw).unwrap_err();
        assert!(matches!(err, TabstructError::InvalidCsvHeader { .. }));
    }

    #[test]
    fn validate_headers_ends_with_dot() {
        let raw = RawCsvTable {
            headers: vec!["a.".into()],
            rows: vec![],
        };
        let err = validate_headers(&raw).unwrap_err();
        assert!(matches!(err, TabstructError::InvalidCsvHeader { .. }));
    }

    #[test]
    fn validate_headers_double_dot() {
        let raw = RawCsvTable {
            headers: vec!["a..b".into()],
            rows: vec![],
        };
        let err = validate_headers(&raw).unwrap_err();
        assert!(matches!(err, TabstructError::InvalidCsvHeader { .. }));
    }

    #[test]
    fn validate_headers_duplicate() {
        let raw = RawCsvTable {
            headers: vec!["a".into(), "b".into(), "a".into()],
            rows: vec![],
        };
        let err = validate_headers(&raw).unwrap_err();
        assert!(matches!(err, TabstructError::DuplicateCsvHeader { .. }));
    }

    #[test]
    fn validate_path_conflict_settings_and_interval() {
        let headers = vec!["settings".into(), "settings.interval".into()];
        let err = validate_path_conflicts(&headers).unwrap_err();
        assert!(matches!(err, TabstructError::PathConflict { path } if path == "settings"));
    }

    #[test]
    fn infer_cell_type_boolean() {
        assert_eq!(infer_cell_type("true"), Some(ColumnType::Boolean));
        assert_eq!(infer_cell_type("false"), Some(ColumnType::Boolean));
    }

    #[test]
    fn infer_cell_type_integer() {
        assert_eq!(infer_cell_type("0"), Some(ColumnType::Integer));
        assert_eq!(infer_cell_type("42"), Some(ColumnType::Integer));
        assert_eq!(infer_cell_type("-1"), Some(ColumnType::Integer));
    }

    #[test]
    fn infer_cell_type_float() {
        assert_eq!(infer_cell_type("3.14"), Some(ColumnType::Float));
        assert_eq!(infer_cell_type("1.0"), Some(ColumnType::Float));
    }

    #[test]
    fn infer_cell_type_string() {
        assert_eq!(infer_cell_type("abc"), Some(ColumnType::String));
        assert_eq!(infer_cell_type(""), None);
    }

    #[test]
    fn merge_column_type_integer_float_becomes_float() {
        assert_eq!(
            merge_column_type(Some(ColumnType::Integer), Some(ColumnType::Float)),
            Some(ColumnType::Float)
        );
        assert_eq!(
            merge_column_type(Some(ColumnType::Float), Some(ColumnType::Integer)),
            Some(ColumnType::Float)
        );
    }

    #[test]
    fn merge_column_type_mixed_becomes_string() {
        assert_eq!(
            merge_column_type(Some(ColumnType::Boolean), Some(ColumnType::Integer)),
            Some(ColumnType::String)
        );
        assert_eq!(
            merge_column_type(Some(ColumnType::Integer), Some(ColumnType::String)),
            Some(ColumnType::String)
        );
    }

    #[test]
    fn typed_boolean_column() {
        let raw = parse_csv("flag\ntrue\nfalse").unwrap();
        let typed = raw_to_typed(raw).unwrap();
        assert_eq!(typed.column_types[0].kind, PrimitiveKind::Boolean);
        assert!(!typed.column_types[0].nullable);
        assert_eq!(typed.rows[0][0], DataValue::Bool(true));
        assert_eq!(typed.rows[1][0], DataValue::Bool(false));
    }

    #[test]
    fn typed_integer_column() {
        let raw = parse_csv("id\n1\n2\n3").unwrap();
        let typed = raw_to_typed(raw).unwrap();
        assert_eq!(typed.column_types[0].kind, PrimitiveKind::Integer);
    }

    #[test]
    fn typed_float_column() {
        let raw = parse_csv("x\n1.0\n2.5").unwrap();
        let typed = raw_to_typed(raw).unwrap();
        assert_eq!(typed.column_types[0].kind, PrimitiveKind::Float);
    }

    #[test]
    fn typed_string_column() {
        let raw = parse_csv("name\nhello\nworld").unwrap();
        let typed = raw_to_typed(raw).unwrap();
        assert_eq!(typed.column_types[0].kind, PrimitiveKind::String);
    }

    #[test]
    fn typed_nullable_column() {
        // 2列目で空セルを混ぜる（カンマで空を明示）
        let raw = parse_csv("a,b\n1,2\n,2\n3,4").unwrap();
        let typed = raw_to_typed(raw).unwrap();
        assert!(
            typed.column_types[0].nullable,
            "first column has empty cell"
        );
        assert_eq!(typed.rows[0][0], DataValue::Integer(1));
        assert_eq!(typed.rows[1][0], DataValue::Null);
        assert_eq!(typed.rows[2][0], DataValue::Integer(3));
    }

    #[test]
    fn typed_all_empty_column_is_string_nullable() {
        // 全セル空の列は string? 扱い
        let raw = parse_csv("empty\n\n\n").unwrap();
        let typed = raw_to_typed(raw).unwrap();
        assert_eq!(typed.column_types[0].kind, PrimitiveKind::String);
        assert!(typed.column_types[0].nullable);
    }

    #[test]
    fn typed_mixed_becomes_string() {
        let raw = parse_csv("mixed\ntrue\n42\nhello").unwrap();
        let typed = raw_to_typed(raw).unwrap();
        assert_eq!(typed.column_types[0].kind, PrimitiveKind::String);
    }

    // CSV → Document 統合テストは converter::csv_to_model::tests および
    // app 経由の get_document(CSV) で実施する。
}
