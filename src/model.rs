use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum DataValue {
    Null,
    Bool(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Array(Vec<DataValue>),
    Object(BTreeMap<String, DataValue>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputFormat {
    Csv,
    Json,
    Yaml,
}

/// ルートのデータ形状（object または array）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RootType {
    Object,
    Array,
}

#[derive(Debug, Clone)]
pub struct Document {
    pub format: InputFormat,
    pub root: DataValue,
}

