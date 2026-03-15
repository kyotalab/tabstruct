//! 内部モデル (Document) -> CSV への変換。
//!
//! ## validate の流れ
//! 1. `validate_csv_compatible_root(root)` でルートを検証。
//! 2. `Object` → その1件をレコードとして採用。
//! 3. `Array` → 各要素が `Object` であることを確認し、1要素＝1レコード。非 Object なら `NonObjectArrayElement`。
//! 4. 上記以外（String, Number, Array など）→ `InvalidCsvRoot`。
//!
//! ## flatten の流れ
//! 1. 各レコード（Object）に対して `flatten_object(value, None, &mut row)` を呼ぶ。
//! 2. `Object` は再帰し、キーを `prefix.key` でつなげる（例: `settings.interval`）。
//! 3. スカラ（Bool/Integer/Float/String）は `path -> 文字列` で row に挿入。
//! 4. `Null` は空文字で挿入（空セル）。
//! 5. `Array` を見つけたら即 `ArrayNotSupportedForCsv` でエラー。
//!
//! ## ヘッダ順の決め方
//! 全レコードを flatten したあと、全 row のキーを集約し **辞書順（BTreeSet）** でソート。
//! これで JSON/YAML 起点でもヘッダ順が安定する。

use std::collections::BTreeMap;

use crate::error::TabstructError;
use crate::model::{DataValue, Document};

/// CSV 変換可能なルートか検証し、オブジェクトマップのスライスを返す。
/// - root が Object → その1件
/// - root が Array で全要素が Object → 各要素
/// - それ以外 → Err
pub fn validate_csv_compatible_root(
    root: &DataValue,
) -> Result<Vec<&BTreeMap<String, DataValue>>, TabstructError> {
    match root {
        DataValue::Object(map) => Ok(vec![map]),
        DataValue::Array(items) => {
            let mut out = Vec::with_capacity(items.len());
            for (idx, item) in items.iter().enumerate() {
                match item {
                    DataValue::Object(map) => out.push(map),
                    _ => {
                        return Err(TabstructError::NonObjectArrayElement { index: idx });
                    }
                }
            }
            Ok(out)
        }
        other => Err(TabstructError::InvalidCsvRoot {
            found: format!("{other:?}"),
        }),
    }
}

/// 1オブジェクトを path -> 文字列 に flatten する。
/// ネストオブジェクトは `a.b.c` 形式のキーにし、Array があればエラーにする。
/// Null は空文字として out に載せる。
pub fn flatten_object(
    value: &DataValue,
    prefix: Option<&str>,
    out: &mut BTreeMap<String, String>,
) -> Result<(), TabstructError> {
    match value {
        DataValue::Object(map) => {
            for (k, v) in map {
                let next = match prefix {
                    Some(p) => format!("{p}.{k}"),
                    None => k.clone(),
                };
                flatten_object(v, Some(&next), out)?;
            }
        }
        DataValue::Array(_) => {
            return Err(TabstructError::ArrayNotSupportedForCsv {
                path: prefix.unwrap_or("<root>").to_string(),
            });
        }
        DataValue::Null => {
            let key = prefix.unwrap_or("<root>").to_string();
            out.insert(key, String::new());
        }
        DataValue::Bool(b) => {
            let key = prefix.unwrap_or("<root>").to_string();
            out.insert(key, b.to_string());
        }
        DataValue::Integer(i) => {
            let key = prefix.unwrap_or("<root>").to_string();
            out.insert(key, i.to_string());
        }
        DataValue::Float(f) => {
            let key = prefix.unwrap_or("<root>").to_string();
            out.insert(key, f.to_string());
        }
        DataValue::String(s) => {
            let key = prefix.unwrap_or("<root>").to_string();
            out.insert(key, s.clone());
        }
    }
    Ok(())
}

/// 全レコードから leaf path を収集し、JSON/YAML 起点では辞書順で安定化したヘッダリストを返す。
fn collect_headers_sorted(records: &[BTreeMap<String, String>]) -> Vec<String> {
    let mut set = std::collections::BTreeSet::new();
    for row in records {
        for k in row.keys() {
            set.insert(k.clone());
        }
    }
    set.into_iter().collect()
}

/// Document の root を CSV 文字列（ヘッダ + 行）にシリアライズする。
/// 事前に validate し、flatten してから csv crate で出力する。
pub fn document_to_csv(doc: &Document) -> Result<String, TabstructError> {
    let record_maps = validate_csv_compatible_root(&doc.root)?;

    let mut rows: Vec<BTreeMap<String, String>> = Vec::with_capacity(record_maps.len());
    for map in &record_maps {
        let obj = DataValue::Object((*map).clone());
        let mut row = BTreeMap::new();
        flatten_object(&obj, None, &mut row)?;
        rows.push(row);
    }

    let headers = collect_headers_sorted(&rows);

    let mut wtr = csv::Writer::from_writer(Vec::new());
    wtr.write_record(&headers)
        .map_err(|e| TabstructError::IoWrite {
            message: e.to_string(),
        })?;
    for row in &rows {
        let values: Vec<&str> = headers
            .iter()
            .map(|h| row.get(h).map(|s| s.as_str()).unwrap_or(""))
            .collect();
        wtr.write_record(&values)
            .map_err(|e| TabstructError::IoWrite {
                message: e.to_string(),
            })?;
    }
    let bytes = wtr.into_inner().map_err(|e| TabstructError::IoWrite {
        message: e.to_string(),
    })?;
    String::from_utf8(bytes).map_err(|e| TabstructError::IoWrite {
        message: e.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{DataValue, Document, InputFormat};
    use std::collections::BTreeMap;

    #[test]
    fn validate_root_object() {
        let map = BTreeMap::new();
        let root = DataValue::Object(map);
        let records = validate_csv_compatible_root(&root).unwrap();
        assert_eq!(records.len(), 1);
    }

    #[test]
    fn validate_root_array_of_objects() {
        let a = BTreeMap::new();
        let b = BTreeMap::new();
        let root = DataValue::Array(vec![DataValue::Object(a), DataValue::Object(b)]);
        let records = validate_csv_compatible_root(&root).unwrap();
        assert_eq!(records.len(), 2);
    }

    #[test]
    fn validate_root_array_with_non_object_errors() {
        let root = DataValue::Array(vec![
            DataValue::Object(BTreeMap::new()),
            DataValue::Integer(1),
        ]);
        let err = validate_csv_compatible_root(&root).unwrap_err();
        assert!(matches!(
            err,
            TabstructError::NonObjectArrayElement { index: 1 }
        ));
    }

    #[test]
    fn validate_root_invalid_root_errors() {
        let root = DataValue::String("x".to_string());
        let err = validate_csv_compatible_root(&root).unwrap_err();
        assert!(matches!(err, TabstructError::InvalidCsvRoot { .. }));
    }

    #[test]
    fn flatten_object_simple() {
        let mut map = BTreeMap::new();
        map.insert("a".to_string(), DataValue::Integer(1));
        map.insert("b".to_string(), DataValue::String("x".to_string()));
        let value = DataValue::Object(map);
        let mut out = BTreeMap::new();
        flatten_object(&value, None, &mut out).unwrap();
        assert_eq!(out.get("a"), Some(&"1".to_string()));
        assert_eq!(out.get("b"), Some(&"x".to_string()));
    }

    #[test]
    fn flatten_object_nested() {
        let mut inner = BTreeMap::new();
        inner.insert("interval".to_string(), DataValue::Integer(5));
        inner.insert(
            "url".to_string(),
            DataValue::String("https://example.com".to_string()),
        );
        let mut map = BTreeMap::new();
        map.insert("id".to_string(), DataValue::Integer(1));
        map.insert(
            "name".to_string(),
            DataValue::String("canary-a".to_string()),
        );
        map.insert("settings".to_string(), DataValue::Object(inner));
        let value = DataValue::Object(map);
        let mut out = BTreeMap::new();
        flatten_object(&value, None, &mut out).unwrap();
        assert_eq!(out.get("id"), Some(&"1".to_string()));
        assert_eq!(out.get("name"), Some(&"canary-a".to_string()));
        assert_eq!(out.get("settings.interval"), Some(&"5".to_string()));
        assert_eq!(
            out.get("settings.url"),
            Some(&"https://example.com".to_string())
        );
    }

    #[test]
    fn flatten_object_null_becomes_empty_cell() {
        let mut map = BTreeMap::new();
        map.insert("a".to_string(), DataValue::Null);
        map.insert("b".to_string(), DataValue::String("x".to_string()));
        let value = DataValue::Object(map);
        let mut out = BTreeMap::new();
        flatten_object(&value, None, &mut out).unwrap();
        assert_eq!(out.get("a"), Some(&"".to_string()));
        assert_eq!(out.get("b"), Some(&"x".to_string()));
    }

    #[test]
    fn flatten_object_array_errors() {
        let mut map = BTreeMap::new();
        map.insert(
            "targets".to_string(),
            DataValue::Array(vec![
                DataValue::String("a".to_string()),
                DataValue::String("b".to_string()),
            ]),
        );
        let value = DataValue::Object(map);
        let mut out = BTreeMap::new();
        let err = flatten_object(&value, None, &mut out).unwrap_err();
        assert!(
            matches!(err, TabstructError::ArrayNotSupportedForCsv { path } if path == "targets")
        );
    }

    #[test]
    fn document_to_csv_single_object() {
        let mut map = BTreeMap::new();
        map.insert("id".to_string(), DataValue::Integer(1));
        map.insert(
            "name".to_string(),
            DataValue::String("canary-a".to_string()),
        );
        let doc = Document {
            format: InputFormat::Json,
            root: DataValue::Object(map),
        };
        let csv = document_to_csv(&doc).unwrap();
        let lines: Vec<&str> = csv.lines().collect();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "id,name");
        assert_eq!(lines[1], "1,canary-a");
    }

    #[test]
    fn document_to_csv_array_of_objects() {
        let mut m1 = BTreeMap::new();
        m1.insert("id".to_string(), DataValue::Integer(1));
        m1.insert("name".to_string(), DataValue::String("a".to_string()));
        let mut m2 = BTreeMap::new();
        m2.insert("id".to_string(), DataValue::Integer(2));
        m2.insert("name".to_string(), DataValue::String("b".to_string()));
        let doc = Document {
            format: InputFormat::Json,
            root: DataValue::Array(vec![DataValue::Object(m1), DataValue::Object(m2)]),
        };
        let csv = document_to_csv(&doc).unwrap();
        let lines: Vec<&str> = csv.lines().collect();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "id,name");
        assert_eq!(lines[1], "1,a");
        assert_eq!(lines[2], "2,b");
    }

    #[test]
    fn document_to_csv_nested_object() {
        let mut inner = BTreeMap::new();
        inner.insert("interval".to_string(), DataValue::Integer(5));
        inner.insert(
            "url".to_string(),
            DataValue::String("https://example.com".to_string()),
        );
        let mut map = BTreeMap::new();
        map.insert("id".to_string(), DataValue::Integer(1));
        map.insert(
            "name".to_string(),
            DataValue::String("canary-a".to_string()),
        );
        map.insert("settings".to_string(), DataValue::Object(inner));
        let doc = Document {
            format: InputFormat::Json,
            root: DataValue::Object(map),
        };
        let csv = document_to_csv(&doc).unwrap();
        let lines: Vec<&str> = csv.lines().collect();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "id,name,settings.interval,settings.url");
        assert_eq!(lines[1], "1,canary-a,5,https://example.com");
    }

    #[test]
    fn document_to_csv_null_empty_cell() {
        let mut map = BTreeMap::new();
        map.insert("a".to_string(), DataValue::Integer(1));
        map.insert("b".to_string(), DataValue::Null);
        map.insert("c".to_string(), DataValue::String("x".to_string()));
        let doc = Document {
            format: InputFormat::Json,
            root: DataValue::Object(map),
        };
        let csv = document_to_csv(&doc).unwrap();
        let lines: Vec<&str> = csv.lines().collect();
        assert_eq!(lines[0], "a,b,c");
        assert_eq!(lines[1], "1,,x");
    }

    #[test]
    fn document_to_csv_root_array_with_non_object_errors() {
        let doc = Document {
            format: InputFormat::Json,
            root: DataValue::Array(vec![
                DataValue::Object(BTreeMap::new()),
                DataValue::String("not object".to_string()),
            ]),
        };
        let err = document_to_csv(&doc).unwrap_err();
        assert!(matches!(
            err,
            TabstructError::NonObjectArrayElement { index: 1 }
        ));
    }

    #[test]
    fn document_to_csv_root_not_object_or_array_errors() {
        let doc = Document {
            format: InputFormat::Json,
            root: DataValue::String("only string".to_string()),
        };
        let err = document_to_csv(&doc).unwrap_err();
        assert!(matches!(err, TabstructError::InvalidCsvRoot { .. }));
    }

    #[test]
    fn document_to_csv_array_field_errors() {
        let mut map = BTreeMap::new();
        map.insert("name".to_string(), DataValue::String("test".to_string()));
        map.insert(
            "targets".to_string(),
            DataValue::Array(vec![
                DataValue::String("a".to_string()),
                DataValue::String("b".to_string()),
            ]),
        );
        let doc = Document {
            format: InputFormat::Json,
            root: DataValue::Object(map),
        };
        let err = document_to_csv(&doc).unwrap_err();
        assert!(matches!(
            err,
            TabstructError::ArrayNotSupportedForCsv { .. }
        ));
    }

    #[test]
    fn headers_stable_sorted() {
        let mut m1 = BTreeMap::new();
        m1.insert("z".to_string(), DataValue::Integer(1));
        m1.insert("a".to_string(), DataValue::Integer(2));
        let mut m2 = BTreeMap::new();
        m2.insert("a".to_string(), DataValue::Integer(3));
        m2.insert("m".to_string(), DataValue::Integer(4));
        m2.insert("z".to_string(), DataValue::Integer(5));
        let doc = Document {
            format: InputFormat::Yaml,
            root: DataValue::Array(vec![DataValue::Object(m1), DataValue::Object(m2)]),
        };
        let csv = document_to_csv(&doc).unwrap();
        let first_line = csv.lines().next().unwrap();
        assert_eq!(first_line, "a,m,z");
    }
}
