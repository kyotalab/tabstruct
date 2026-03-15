//! schema 出力などの整形用モジュール。

use crate::model::{InputFormat, RootType};
use crate::schema::SchemaReport;

/// 形式を schema 表示用の文字列にする（大文字表記）。
fn format_label(format: InputFormat) -> &'static str {
    match format {
        InputFormat::Csv => "CSV",
        InputFormat::Json => "JSON",
        InputFormat::Yaml => "YAML",
    }
}

/// ルート型を schema 表示用の文字列にする（小文字）。
fn root_type_label(root_type: RootType) -> &'static str {
    match root_type {
        RootType::Object => "object",
        RootType::Array => "array",
    }
}

/// SchemaReport を仕様どおりのテキストに整形する。
///
/// 出力形式:
/// - Format: CSV|JSON|YAML
/// - Root Type: object|array
/// - Records: N
/// - 空行
/// - Fields:
/// - path: type?
pub fn format_schema_report(report: &SchemaReport) -> String {
    let format_str = format_label(report.format);
    let root_str = root_type_label(report.root_type);

    let mut out = format!(
        "Format: {format_str}\nRoot Type: {root_str}\nRecords: {}\n\nFields:\n",
        report.records
    );

    for field in &report.fields {
        let type_str = field.field_type.to_display_str();
        out.push_str(&format!("- {}: {type_str}\n", field.path));
    }

    out.push('\n');
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::types::{DisplayType, PrimitiveKind, SchemaField};

    #[test]
    fn format_schema_report_basic() {
        let report = SchemaReport {
            format: InputFormat::Json,
            root_type: RootType::Array,
            records: 2,
            fields: vec![
                SchemaField {
                    path: "id".to_string(),
                    field_type: DisplayType {
                        kind: PrimitiveKind::Integer,
                        nullable: false,
                    },
                },
                SchemaField {
                    path: "name".to_string(),
                    field_type: DisplayType {
                        kind: PrimitiveKind::String,
                        nullable: true,
                    },
                },
            ],
        };
        let text = format_schema_report(&report);
        assert!(text.contains("Format: JSON"));
        assert!(text.contains("Root Type: array"));
        assert!(text.contains("Records: 2"));
        assert!(text.contains("Fields:"));
        assert!(text.contains("- id: integer"));
        assert!(text.contains("- name: string?"));
    }

    #[test]
    fn format_schema_report_yaml_object() {
        let report = SchemaReport {
            format: InputFormat::Yaml,
            root_type: RootType::Object,
            records: 1,
            fields: vec![SchemaField {
                path: "enabled".to_string(),
                field_type: DisplayType {
                    kind: PrimitiveKind::Boolean,
                    nullable: false,
                },
            }],
        };
        let text = format_schema_report(&report);
        assert!(text.contains("Format: YAML"));
        assert!(text.contains("Root Type: object"));
        assert!(text.contains("Records: 1"));
        assert!(text.contains("- enabled: boolean"));
    }
}
