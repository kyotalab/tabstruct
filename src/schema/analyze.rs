//! Schema 解析: Document から SchemaReport を生成する。
//! CSV/JSON/YAML いずれも、Document の root が Object または Array であれば対応。

use crate::error::TabstructError;
use crate::model::{DataValue, Document, RootType};
use crate::schema::types::{DisplayType, PrimitiveKind, SchemaField, SchemaReport};
use std::collections::BTreeMap;

/// ルートが Object の場合はその1件、Array の場合は各要素を返す。
/// CSV やルートがスカラーの場合はエラー。
fn records_from_root(root: &DataValue) -> Result<Vec<&DataValue>, TabstructError> {
    match root {
        DataValue::Object(_) => Ok(vec![root]),
        DataValue::Array(arr) => Ok(arr.iter().collect()),
        _ => Err(TabstructError::internal(
            "schema: root must be object or array",
        )),
    }
}

/// 1つの値（Object またはスカラー等）を走査し、leaf path -> 観測値のリストを out に追加する。
/// prefix はドット区切りパス。Object の子は再帰し、それ以外は leaf として path に追加する。
pub fn collect_leaf_paths(
    value: &DataValue,
    prefix: Option<&str>,
    out: &mut BTreeMap<String, Vec<DataValue>>,
) {
    match value {
        DataValue::Object(map) => {
            for (k, v) in map {
                let next = match prefix {
                    Some(p) => format!("{p}.{k}"),
                    None => k.clone(),
                };
                collect_leaf_paths(v, Some(&next), out);
            }
        }
        leaf => {
            let path = prefix.unwrap_or("<root>").to_string();
            out.entry(path).or_default().push(leaf.clone());
        }
    }
}

/// 同一 path で観測した値のリストから表示型を集約する。
/// - null が1つでもあれば nullable = true
/// - integer + float は float
/// - 異なる primitive が混在すれば Mixed
pub fn infer_display_type(values: &[DataValue]) -> DisplayType {
    let mut nullable = false;
    let mut current: Option<PrimitiveKind> = None;

    for v in values {
        let next = match v {
            DataValue::Null => {
                nullable = true;
                continue;
            }
            DataValue::Bool(_) => PrimitiveKind::Boolean,
            DataValue::Integer(_) => PrimitiveKind::Integer,
            DataValue::Float(_) => PrimitiveKind::Float,
            DataValue::String(_) => PrimitiveKind::String,
            DataValue::Object(_) => PrimitiveKind::Object,
            DataValue::Array(_) => PrimitiveKind::Array,
        };

        current = Some(match (current, next) {
            (None, k) => k,
            (Some(PrimitiveKind::Integer), PrimitiveKind::Float)
            | (Some(PrimitiveKind::Float), PrimitiveKind::Integer) => PrimitiveKind::Float,
            (Some(a), b) if a == b => a,
            _ => PrimitiveKind::Mixed,
        });
    }

    DisplayType {
        kind: current.unwrap_or(PrimitiveKind::String),
        nullable,
    }
}

/// Document から SchemaReport を生成する。CSV/JSON/YAML いずれも、root が Object または Array であれば対応。
pub fn analyze(doc: &Document) -> Result<SchemaReport, TabstructError> {
    let root_type = match &doc.root {
        DataValue::Object(_) => RootType::Object,
        DataValue::Array(_) => RootType::Array,
        _ => {
            return Err(TabstructError::internal(
                "schema: root must be object or array",
            ));
        }
    };

    let records = records_from_root(&doc.root)?;
    let records_count = records.len();

    let mut path_to_values: BTreeMap<String, Vec<DataValue>> = BTreeMap::new();
    for record in &records {
        collect_leaf_paths(record, None, &mut path_to_values);
    }

    // "<root>" はルートがスカラー等のときのみ出る。object/array のときは不要なので除外する。
    let fields: Vec<SchemaField> = path_to_values
        .into_iter()
        .filter(|(path, _)| path != "<root>")
        .map(|(path, values)| {
            let field_type = infer_display_type(&values);
            SchemaField { path, field_type }
        })
        .collect();

    Ok(SchemaReport {
        format: doc.format,
        root_type,
        records: records_count,
        fields,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{DataValue, InputFormat};

    #[test]
    fn collect_leaf_paths_single_object() {
        let obj = DataValue::Object(
            [
                ("id".to_string(), DataValue::Integer(1)),
                ("name".to_string(), DataValue::String("a".into())),
            ]
            .into_iter()
            .collect(),
        );
        let mut out = BTreeMap::new();
        collect_leaf_paths(&obj, None, &mut out);
        assert_eq!(out.len(), 2);
        assert_eq!(out.get("id").map(|v| v.len()), Some(1));
        assert_eq!(out.get("name").map(|v| v.len()), Some(1));
    }

    #[test]
    fn collect_leaf_paths_nested() {
        let obj = DataValue::Object(
            [(
                "settings".to_string(),
                DataValue::Object(
                    [
                        ("interval".to_string(), DataValue::Integer(5)),
                        ("url".to_string(), DataValue::String("https://x.com".into())),
                    ]
                    .into_iter()
                    .collect(),
                ),
            )]
            .into_iter()
            .collect(),
        );
        let mut out = BTreeMap::new();
        collect_leaf_paths(&obj, None, &mut out);
        assert!(out.contains_key("settings.interval"));
        assert!(out.contains_key("settings.url"));
    }

    #[test]
    fn infer_display_type_integer() {
        let values = vec![DataValue::Integer(1), DataValue::Integer(2)];
        let dt = infer_display_type(&values);
        assert_eq!(dt.kind, PrimitiveKind::Integer);
        assert!(!dt.nullable);
    }

    #[test]
    fn infer_display_type_integer_and_float() {
        let values = vec![DataValue::Integer(1), DataValue::Float(2.5)];
        let dt = infer_display_type(&values);
        assert_eq!(dt.kind, PrimitiveKind::Float);
    }

    #[test]
    fn infer_display_type_nullable() {
        let values = vec![
            DataValue::String("a".into()),
            DataValue::Null,
            DataValue::String("b".into()),
        ];
        let dt = infer_display_type(&values);
        assert_eq!(dt.kind, PrimitiveKind::String);
        assert!(dt.nullable);
    }

    #[test]
    fn infer_display_type_mixed() {
        let values = vec![DataValue::Integer(1), DataValue::String("x".into())];
        let dt = infer_display_type(&values);
        assert_eq!(dt.kind, PrimitiveKind::Mixed);
    }

    #[test]
    fn infer_display_type_all_null() {
        let values = vec![DataValue::Null, DataValue::Null];
        let dt = infer_display_type(&values);
        assert_eq!(dt.kind, PrimitiveKind::String);
        assert!(dt.nullable);
    }

    #[test]
    fn analyze_json_object() {
        let doc = Document {
            format: InputFormat::Json,
            root: DataValue::Object(
                [
                    ("id".to_string(), DataValue::Integer(1)),
                    ("name".to_string(), DataValue::String("test".into())),
                ]
                .into_iter()
                .collect(),
            ),
        };
        let report = analyze(&doc).unwrap();
        assert_eq!(report.format, InputFormat::Json);
        assert_eq!(report.root_type, RootType::Object);
        assert_eq!(report.records, 1);
        assert_eq!(report.fields.len(), 2);
        let paths: Vec<_> = report.fields.iter().map(|f| f.path.as_str()).collect();
        assert!(paths.contains(&"id"));
        assert!(paths.contains(&"name"));
    }

    #[test]
    fn analyze_json_array() {
        let doc = Document {
            format: InputFormat::Json,
            root: DataValue::Array(vec![
                DataValue::Object(
                    [("id".to_string(), DataValue::Integer(1))]
                        .into_iter()
                        .collect(),
                ),
                DataValue::Object(
                    [("id".to_string(), DataValue::Integer(2))]
                        .into_iter()
                        .collect(),
                ),
            ]),
        };
        let report = analyze(&doc).unwrap();
        assert_eq!(report.root_type, RootType::Array);
        assert_eq!(report.records, 2);
        assert_eq!(report.fields.len(), 1);
        assert_eq!(report.fields[0].path, "id");
    }

    #[test]
    fn analyze_yaml_object() {
        let doc = Document {
            format: InputFormat::Yaml,
            root: DataValue::Object(
                [("name".to_string(), DataValue::String("yaml".into()))]
                    .into_iter()
                    .collect(),
            ),
        };
        let report = analyze(&doc).unwrap();
        assert_eq!(report.format, InputFormat::Yaml);
        assert_eq!(report.root_type, RootType::Object);
        assert_eq!(report.records, 1);
    }

    #[test]
    fn analyze_csv_array_of_objects() {
        // CSV 由来の Document（root が Array<Object>）も schema で解析可能
        let doc = Document {
            format: InputFormat::Csv,
            root: DataValue::Array(vec![
                DataValue::Object(
                    [
                        ("id".to_string(), DataValue::Integer(1)),
                        ("name".to_string(), DataValue::String("alice".into())),
                    ]
                    .into_iter()
                    .collect(),
                ),
                DataValue::Object(
                    [
                        ("id".to_string(), DataValue::Integer(2)),
                        ("name".to_string(), DataValue::String("bob".into())),
                    ]
                    .into_iter()
                    .collect(),
                ),
            ]),
        };
        let report = analyze(&doc).unwrap();
        assert_eq!(report.format, InputFormat::Csv);
        assert_eq!(report.root_type, RootType::Array);
        assert_eq!(report.records, 2);
        assert_eq!(report.fields.len(), 2);
        let paths: Vec<_> = report.fields.iter().map(|f| f.path.as_str()).collect();
        assert!(paths.contains(&"id"));
        assert!(paths.contains(&"name"));
    }

    #[test]
    fn analyze_json_array_nullable_field() {
        let doc = Document {
            format: InputFormat::Json,
            root: DataValue::Array(vec![
                DataValue::Object(
                    [
                        ("id".to_string(), DataValue::Integer(1)),
                        ("name".to_string(), DataValue::String("a".into())),
                    ]
                    .into_iter()
                    .collect(),
                ),
                DataValue::Object(
                    [
                        ("id".to_string(), DataValue::Integer(2)),
                        ("name".to_string(), DataValue::Null),
                    ]
                    .into_iter()
                    .collect(),
                ),
            ]),
        };
        let report = analyze(&doc).unwrap();
        let name_field = report.fields.iter().find(|f| f.path == "name").unwrap();
        assert_eq!(name_field.field_type.kind, PrimitiveKind::String);
        assert!(name_field.field_type.nullable);
    }

    #[test]
    fn analyze_json_array_integer_and_float_becomes_float() {
        let doc = Document {
            format: InputFormat::Json,
            root: DataValue::Array(vec![
                DataValue::Object(
                    [("value".to_string(), DataValue::Integer(1))]
                        .into_iter()
                        .collect(),
                ),
                DataValue::Object(
                    [("value".to_string(), DataValue::Float(2.5))]
                        .into_iter()
                        .collect(),
                ),
            ]),
        };
        let report = analyze(&doc).unwrap();
        let value_field = report.fields.iter().find(|f| f.path == "value").unwrap();
        assert_eq!(value_field.field_type.kind, PrimitiveKind::Float);
    }

    #[test]
    fn analyze_json_array_mixed_types() {
        let doc = Document {
            format: InputFormat::Json,
            root: DataValue::Array(vec![
                DataValue::Object(
                    [("x".to_string(), DataValue::Integer(1))]
                        .into_iter()
                        .collect(),
                ),
                DataValue::Object(
                    [("x".to_string(), DataValue::String("text".into()))]
                        .into_iter()
                        .collect(),
                ),
            ]),
        };
        let report = analyze(&doc).unwrap();
        let x_field = report.fields.iter().find(|f| f.path == "x").unwrap();
        assert_eq!(x_field.field_type.kind, PrimitiveKind::Mixed);
    }
}
