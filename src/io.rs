use crate::cli::InputArgs;
use crate::error::TabstructError;
use std::fs;
use std::path::Path;

pub struct InputContent {
    pub path: Option<String>,
    pub content: String,
}

pub fn read_input(_args: &InputArgs) -> Result<InputContent, TabstructError> {
    // 入力処理は後続単位で詳細実装する
    Err(TabstructError::internal(
        "read_input is not implemented yet",
    ))
}

pub fn detect_input_format(
    _args: &InputArgs,
    _path: Option<&Path>,
) -> Result<crate::model::InputFormat, TabstructError> {
    // 形式判定も後続単位で実装
    Err(TabstructError::internal(
        "detect_input_format is not implemented yet",
    ))
}

pub fn write_stdout(_text: &str) -> Result<(), TabstructError> {
    // とりあえず標準出力へ書くだけの実装を後続で追加
    Err(TabstructError::internal(
        "write_stdout is not implemented yet",
    ))
}

pub fn write_output(_path: Option<&std::path::PathBuf>, _content: &str) -> Result<(), TabstructError> {
    // ファイル出力も後続単位で実装
    Err(TabstructError::internal(
        "write_output is not implemented yet",
    ))
}

fn _read_file_to_string(path: &Path) -> Result<String, TabstructError> {
    fs::read_to_string(path).map_err(|e| TabstructError::IoRead {
        message: e.to_string(),
    })
}

