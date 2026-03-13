use crate::model::InputFormat;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RootType {
    Object,
    Array,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplayType {
    pub kind: PrimitiveKind,
    pub nullable: bool,
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

