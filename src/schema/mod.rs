pub mod analyze;
pub mod types;

pub use analyze::{analyze, analyze_csv};
pub use crate::model::RootType;
pub use types::{DisplayType, PrimitiveKind, SchemaField, SchemaReport};

