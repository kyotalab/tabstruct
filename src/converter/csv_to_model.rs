//! CSV → 内部モデル (Document) への変換。
//! TypedCsvTable を受け取り、`a.b.c` ヘッダをネストオブジェクトに戻す unflatten を行う。

use crate::error::TabstructError;
use crate::model::{DataValue, Document, InputFormat};
use crate::parser::csv::TypedCsvTable;
use std::collections::BTreeMap;

/// path segments に従って Object を掘り、leaf に value を挿入する。
/// 既に leaf に値がある場合（path conflict）はエラーを返す。
/// 中間キーが既に scalar 等で存在する場合も PathConflict を返す。
pub fn insert_path(
    root: &mut BTreeMap<String, DataValue>,
    segments: &[&str],
    value: DataValue,
) -> Result<(), TabstructError> {
    if segments.is_empty() {
        return Err(TabstructError::internal("empty path segments"));
    }

    let mut current = root;

    for (idx, segment) in segments.iter().enumerate() {
        let is_leaf = idx == segments.len() - 1;

        if is_leaf {
            if current.contains_key(*segment) {
                return Err(TabstructError::PathConflict {
                    path: segments.join("."),
                });
            }
            current.insert((*segment).to_string(), value.clone());
        } else {
            let entry = current
                .entry((*segment).to_string())
                .or_insert_with(|| DataValue::Object(BTreeMap::new()));

            match entry {
                DataValue::Object(map) => current = map,
                _ => {
                    return Err(TabstructError::PathConflict {
                        path: segments[..=idx].join("."),
                    });
                }
            }
        }
    }

    Ok(())
}

/// 1行分の DataValue 列をヘッダ path に従って 1 つの Object に unflatten する。
fn row_to_object(
    headers: &[String],
    row: &[DataValue],
) -> Result<BTreeMap<String, DataValue>, TabstructError> {
    let mut root = BTreeMap::new();
    for (header, value) in headers.iter().zip(row.iter()) {
        let segments: Vec<&str> = header.split('.').collect();
        insert_path(&mut root, &segments, value.clone())?;
    }
    Ok(root)
}

/// TypedCsvTable から Document を生成する。
/// ルートは常に `Array<Object>`（1行 = 1レコード）。
pub fn typed_table_to_document(typed: TypedCsvTable) -> Result<Document, TabstructError> {
    let mut array = Vec::with_capacity(typed.rows.len());
    for row in typed.rows {
        let obj = row_to_object(&typed.headers, &row)?;
        array.push(DataValue::Object(obj));
    }

    Ok(Document {
        format: InputFormat::Csv,
        root: DataValue::Array(array),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::DataValue;
    use crate::parser::csv::TypedCsvTable;
    use crate::schema::types::{DisplayType, PrimitiveKind};

    fn display_type(kind: PrimitiveKind, nullable: bool) -> DisplayType {
        DisplayType { kind, nullable }
    }

    #[test]
    fn insert_path_single_segment() {
        let mut root = BTreeMap::new();
        insert_path(&mut root, &["id"], DataValue::Integer(1)).unwrap();
        assert_eq!(root.get("id"), Some(&DataValue::Integer(1)));
    }

    #[test]
    fn insert_path_nested_two_levels() {
        let mut root = BTreeMap::new();
        insert_path(&mut root, &["settings", "interval"], DataValue::Integer(5)).unwrap();
        let settings = root.get("settings").and_then(|v| match v {
            DataValue::Object(m) => Some(m),
            _ => None,
        });
        assert!(settings.is_some());
        assert_eq!(
            settings.unwrap().get("interval"),
            Some(&DataValue::Integer(5))
        );
    }

    #[test]
    fn insert_path_three_levels() {
        let mut root = BTreeMap::new();
        insert_path(
            &mut root,
            &["a", "b", "c"],
            DataValue::String("leaf".into()),
        )
        .unwrap();
        let a = root.get("a").and_then(|v| match v {
            DataValue::Object(m) => Some(m),
            _ => None,
        });
        let b = a.unwrap().get("b").and_then(|v| match v {
            DataValue::Object(m) => Some(m),
            _ => None,
        });
        assert_eq!(b.unwrap().get("c"), Some(&DataValue::String("leaf".into())));
    }

    #[test]
    fn insert_path_null_leaf() {
        let mut root = BTreeMap::new();
        insert_path(&mut root, &["opt"], DataValue::Null).unwrap();
        assert_eq!(root.get("opt"), Some(&DataValue::Null));
    }

    #[test]
    fn insert_path_leaf_conflict_returns_error() {
        let mut root = BTreeMap::new();
        insert_path(&mut root, &["k"], DataValue::Integer(1)).unwrap();
        let err = insert_path(&mut root, &["k"], DataValue::Integer(2)).unwrap_err();
        assert!(matches!(err, TabstructError::PathConflict { path } if path == "k"));
    }

    #[test]
    fn insert_path_empty_segments_error() {
        let mut root = BTreeMap::new();
        let err = insert_path(&mut root, &[], DataValue::Null).unwrap_err();
        assert!(matches!(err, TabstructError::Internal { .. }));
    }

    #[test]
    fn typed_table_to_document_single_row_flat() {
        let typed = TypedCsvTable {
            headers: vec!["id".into(), "name".into()],
            column_types: vec![
                display_type(PrimitiveKind::Integer, false),
                display_type(PrimitiveKind::String, false),
            ],
            rows: vec![vec![
                DataValue::Integer(1),
                DataValue::String("alice".into()),
            ]],
        };
        let doc = typed_table_to_document(typed).unwrap();
        let arr = match &doc.root {
            DataValue::Array(a) => a,
            _ => panic!("expected array root"),
        };
        assert_eq!(arr.len(), 1);
        let obj = match &arr[0] {
            DataValue::Object(m) => m,
            _ => panic!("expected object"),
        };
        assert_eq!(obj.get("id"), Some(&DataValue::Integer(1)));
        assert_eq!(obj.get("name"), Some(&DataValue::String("alice".into())));
    }

    #[test]
    fn typed_table_to_document_nested() {
        let typed = TypedCsvTable {
            headers: vec![
                "id".into(),
                "settings.interval".into(),
                "settings.url".into(),
            ],
            column_types: vec![
                display_type(PrimitiveKind::Integer, false),
                display_type(PrimitiveKind::Integer, false),
                display_type(PrimitiveKind::String, false),
            ],
            rows: vec![vec![
                DataValue::Integer(1),
                DataValue::Integer(5),
                DataValue::String("https://example.com".into()),
            ]],
        };
        let doc = typed_table_to_document(typed).unwrap();
        let arr = match &doc.root {
            DataValue::Array(a) => a,
            _ => panic!("expected array"),
        };
        let obj = match &arr[0] {
            DataValue::Object(m) => m,
            _ => panic!("expected object"),
        };
        assert_eq!(obj.get("id"), Some(&DataValue::Integer(1)));
        let settings = obj.get("settings").and_then(|v| match v {
            DataValue::Object(m) => Some(m),
            _ => None,
        });
        assert!(settings.is_some());
        let s = settings.unwrap();
        assert_eq!(s.get("interval"), Some(&DataValue::Integer(5)));
        assert_eq!(
            s.get("url"),
            Some(&DataValue::String("https://example.com".into()))
        );
    }

    #[test]
    fn typed_table_to_document_nullable_cell() {
        let typed = TypedCsvTable {
            headers: vec!["a".into(), "b".into()],
            column_types: vec![
                display_type(PrimitiveKind::Integer, true),
                display_type(PrimitiveKind::String, false),
            ],
            rows: vec![vec![DataValue::Null, DataValue::String("x".into())]],
        };
        let doc = typed_table_to_document(typed).unwrap();
        let arr = match &doc.root {
            DataValue::Array(a) => a,
            _ => panic!("expected array"),
        };
        let obj = match &arr[0] {
            DataValue::Object(m) => m,
            _ => panic!("expected object"),
        };
        assert_eq!(obj.get("a"), Some(&DataValue::Null));
        assert_eq!(obj.get("b"), Some(&DataValue::String("x".into())));
    }

    #[test]
    fn typed_table_to_document_multiple_rows() {
        let typed = TypedCsvTable {
            headers: vec!["id".into()],
            column_types: vec![display_type(PrimitiveKind::Integer, false)],
            rows: vec![
                vec![DataValue::Integer(1)],
                vec![DataValue::Integer(2)],
                vec![DataValue::Integer(3)],
            ],
        };
        let doc = typed_table_to_document(typed).unwrap();
        let arr = match &doc.root {
            DataValue::Array(a) => a,
            _ => panic!("expected array"),
        };
        assert_eq!(arr.len(), 3);
        assert_eq!(
            arr.iter()
                .map(|v| match v {
                    DataValue::Object(m) => m.get("id").cloned(),
                    _ => None,
                })
                .collect::<Vec<_>>(),
            vec![
                Some(DataValue::Integer(1)),
                Some(DataValue::Integer(2)),
                Some(DataValue::Integer(3)),
            ]
        );
    }

    #[test]
    fn typed_table_to_document_empty_rows() {
        let typed = TypedCsvTable {
            headers: vec!["id".into()],
            column_types: vec![display_type(PrimitiveKind::Integer, false)],
            rows: vec![],
        };
        let doc = typed_table_to_document(typed).unwrap();
        let arr = match &doc.root {
            DataValue::Array(a) => a,
            _ => panic!("expected array"),
        };
        assert!(arr.is_empty());
    }
}
