use thiserror::Error;

#[derive(Debug, Error)]
pub enum TabstructError {
    #[error("Either --file or --stdin must be specified")]
    MissingInput,

    #[error("Unsupported file extension: {extension}")]
    UnsupportedExtension { extension: String },

    #[error("Input extension is .{expected} but content could not be parsed as {expected}")]
    InputFormatMismatch { expected: String },

    #[error("Failed to read input: {message}")]
    IoRead { message: String },

    #[error("Failed to write output: {message}")]
    IoWrite { message: String },

    #[error("Invalid CSV header at column {column}: \"{header}\"")]
    InvalidCsvHeader { column: usize, header: String },

    #[error("Duplicate CSV header at column {column}: \"{header}\"")]
    DuplicateCsvHeader { column: usize, header: String },

    #[error("CSV row {row} has {actual} columns but expected {expected}")]
    CsvColumnCountMismatch { row: usize, expected: usize, actual: usize },

    #[error("Path conflict detected at \"{path}\"")]
    PathConflict { path: String },

    #[error("Array field \"{path}\" cannot be converted to CSV")]
    ArrayNotSupportedForCsv { path: String },

    #[error("CSV conversion requires root object or array of objects, but found {found}")]
    InvalidCsvRoot { found: String },

    #[error("Mixed root array contains non-object element at index {index}")]
    NonObjectArrayElement { index: usize },

    #[error("JSON parse error: {message}")]
    JsonParse { message: String },

    #[error("YAML parse error: {message}")]
    YamlParse { message: String },

    #[error("CSV parse error at row {row}: {message}")]
    CsvParse { row: usize, message: String },

    #[error("Internal error: {message}")]
    Internal { message: String },
}

impl TabstructError {
    pub fn internal(message: impl Into<String>) -> Self {
        TabstructError::Internal {
            message: message.into(),
        }
    }
}

