pub mod analyze;
pub mod types;

pub use crate::model::RootType;
pub use analyze::{analyze, analyze_csv};
pub use types::{DisplayType, PrimitiveKind, SchemaField, SchemaReport};
