//! 入出力と形式判定の基盤。
//!
//! - ファイル入力 / 標準入力の読み取り
//! - 拡張子または `--type` による形式判定
//! - 標準出力 / ファイル出力

use crate::cli::{InputArgs, InputType};
use crate::error::TabstructError;
use crate::model::InputFormat;
use std::io::{self, Read, Write};
use std::path::Path;

/// 読み取った入力の内容と、入力元パス（ファイルの場合のみ）。
#[derive(Debug, Clone)]
pub struct InputContent {
    /// ファイル入力の場合は `Some(パス文字列)`、stdin の場合は `None`。
    pub path: Option<String>,
    /// 生の入力文字列（UTF-8）。
    pub content: String,
}

/// サポートするファイル拡張子と形式の対応。
pub const EXT_CSV: &str = "csv";
pub const EXT_JSON: &str = "json";
pub const EXT_YAML: &str = "yaml";
pub const EXT_YML: &str = "yml";

/// ファイルから入力する。
pub fn read_input(args: &InputArgs) -> Result<InputContent, TabstructError> {
    if let Some(ref path) = args.file {
        let content = read_file(path.as_path())?;
        let path_str = path.to_string_lossy().into_owned();
        return Ok(InputContent {
            path: Some(path_str),
            content,
        });
    }
    if args.stdin {
        let content = read_stdin()?;
        return Ok(InputContent {
            path: None,
            content,
        });
    }
    Err(TabstructError::MissingInput)
}

/// 拡張子から入力形式を判定する。サポート外の拡張子の場合はエラー。
pub fn format_from_extension(path: &Path) -> Result<InputFormat, TabstructError> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase())
        .ok_or_else(|| TabstructError::UnsupportedExtension {
            extension: path
                .extension()
                .map(|e| format!(".{}", e.to_string_lossy()))
                .unwrap_or_else(|| "（なし）".to_string()),
        })?;

    match ext.as_str() {
        EXT_CSV => Ok(InputFormat::Csv),
        EXT_JSON => Ok(InputFormat::Json),
        EXT_YAML | EXT_YML => Ok(InputFormat::Yaml),
        _ => Err(TabstructError::UnsupportedExtension {
            extension: format!(".{ext}"),
        }),
    }
}

/// 入力元に応じて形式を決定する。
///
/// - stdin の場合: `args.r#type` を必須として使用（clap で `--stdin` 時は `--type` 必須）。
/// - ファイルの場合: 拡張子から判定。サポート外拡張子はエラー。
pub fn detect_input_format(
    args: &InputArgs,
    path: Option<&Path>,
) -> Result<InputFormat, TabstructError> {
    if path.is_none() {
        // stdin
        let t = args.r#type.ok_or_else(|| TabstructError::Internal {
            message: "--type is required when using --stdin".to_string(),
        })?;
        return Ok(input_type_to_format(t));
    }
    let p = path.unwrap();
    format_from_extension(p)
}

/// ファイルを UTF-8 で読み込む。
fn read_file(path: &Path) -> Result<String, TabstructError> {
    std::fs::read_to_string(path).map_err(|e| TabstructError::IoRead {
        message: e.to_string(),
    })
}

/// 標準入力から末尾まで読み込む。
fn read_stdin() -> Result<String, TabstructError> {
    let mut buf = String::new();
    io::stdin()
        .read_to_string(&mut buf)
        .map_err(|e| TabstructError::IoRead {
            message: e.to_string(),
        })?;
    Ok(buf)
}

/// 標準出力へ文字列を書き込む。
pub fn write_stdout(text: &str) -> Result<(), TabstructError> {
    io::stdout()
        .write_all(text.as_bytes())
        .map_err(|e| TabstructError::IoWrite {
            message: e.to_string(),
        })?;
    io::stdout().flush().map_err(|e| TabstructError::IoWrite {
        message: e.to_string(),
    })?;
    Ok(())
}

/// 出力先に応じて書き込む。`path` が `None` の場合は標準出力へ書き込む。
pub fn write_output(path: Option<&std::path::Path>, content: &str) -> Result<(), TabstructError> {
    match path {
        Some(p) => {
            std::fs::write(p, content).map_err(|e| TabstructError::IoWrite {
                message: e.to_string(),
            })?;
        }
        None => write_stdout(content)?,
    }
    Ok(())
}

fn input_type_to_format(t: InputType) -> InputFormat {
    match t {
        InputType::Csv => InputFormat::Csv,
        InputType::Json => InputFormat::Json,
        InputType::Yaml => InputFormat::Yaml,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::{InputArgs, InputType};

    #[test]
    fn format_from_extension_csv() {
        let f = format_from_extension(Path::new("a.csv")).unwrap();
        assert!(matches!(f, InputFormat::Csv));
    }

    #[test]
    fn format_from_extension_json() {
        let f = format_from_extension(Path::new("b.json")).unwrap();
        assert!(matches!(f, InputFormat::Json));
    }

    #[test]
    fn format_from_extension_yaml_yml() {
        assert!(matches!(
            format_from_extension(Path::new("c.yaml")).unwrap(),
            InputFormat::Yaml
        ));
        assert!(matches!(
            format_from_extension(Path::new("d.yml")).unwrap(),
            InputFormat::Yaml
        ));
    }

    #[test]
    fn format_from_extension_unsupported() {
        let err = format_from_extension(Path::new("e.txt")).unwrap_err();
        match &err {
            TabstructError::UnsupportedExtension { extension } => {
                assert_eq!(extension, ".txt");
            }
            _ => panic!("expected UnsupportedExtension"),
        }
    }

    #[test]
    fn format_from_extension_no_extension() {
        let err = format_from_extension(Path::new("noext")).unwrap_err();
        match &err {
            TabstructError::UnsupportedExtension { .. } => {}
            _ => panic!("expected UnsupportedExtension"),
        }
    }

    /// ファイル入力: 一時ファイルを読み取り、形式判定ができること。
    #[test]
    fn read_input_from_file_and_detect_format() {
        let dir = std::env::temp_dir();
        let path = dir.join("tabstruct_io_test_read.csv");
        let content = "a,b\n1,2";
        std::fs::write(&path, content).unwrap();
        let args = InputArgs {
            file: Some(path.clone()),
            stdin: false,
            r#type: None,
        };
        let input = read_input(&args).unwrap();
        assert_eq!(input.content, content);
        assert!(input.path.is_some());
        let path_ref = input.path.as_deref().map(Path::new);
        let format = detect_input_format(&args, path_ref).unwrap();
        assert!(matches!(format, InputFormat::Csv));
        std::fs::remove_file(&path).ok();
    }

    /// stdin 時は --type で形式が決まること。
    #[test]
    fn detect_format_from_stdin_uses_type() {
        let args = InputArgs {
            file: None,
            stdin: true,
            r#type: Some(InputType::Json),
        };
        let format = detect_input_format(&args, None).unwrap();
        assert!(matches!(format, InputFormat::Json));

        let args_yaml = InputArgs {
            file: None,
            stdin: true,
            r#type: Some(InputType::Yaml),
        };
        let format_yaml = detect_input_format(&args_yaml, None).unwrap();
        assert!(matches!(format_yaml, InputFormat::Yaml));
    }

    #[test]
    fn read_input_missing_input_error() {
        let args = InputArgs {
            file: None,
            stdin: false,
            r#type: None,
        };
        let err = read_input(&args).unwrap_err();
        match &err {
            TabstructError::MissingInput => {}
            _ => panic!("expected MissingInput"),
        }
    }
}
