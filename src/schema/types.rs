use crate::model::{InputFormat, RootType};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplayType {
    pub kind: PrimitiveKind,
    pub nullable: bool,
}

impl DisplayType {
    /// 表示用の型文字列を返す。nullable の場合は `?` を付与する。
    /// 例: "integer", "string?"
    pub fn to_display_str(&self) -> String {
        let base = match self.kind {
            PrimitiveKind::Boolean => "boolean",
            PrimitiveKind::Integer => "integer",
            PrimitiveKind::Float => "float",
            PrimitiveKind::String => "string",
            PrimitiveKind::Object => "object",
            PrimitiveKind::Array => "array",
            PrimitiveKind::Mixed => "mixed",
        };
        if self.nullable {
            format!("{base}?")
        } else {
            base.to_string()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrimitiveKind {
    Boolean,
    Integer,
    Float,
    String,
    Object,
    Array,
    Mixed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaField {
    pub path: String,
    pub field_type: DisplayType,
}

#[derive(Debug, Clone)]
pub struct SchemaReport {
    pub format: InputFormat,
    pub root_type: RootType,
    pub records: usize,
    pub fields: Vec<SchemaField>,
}
