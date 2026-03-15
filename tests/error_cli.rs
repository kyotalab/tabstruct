//! 異常系の CLI 結合テスト。
//!
//! 仕様カバー:
//! - stdin 時は --type 必須（省略でエラー）
//! - 不正拡張子はエラー
//! - 拡張子と内容不一致はエラー
//! - 不正ヘッダはエラー
//! - array field の CSV 変換はエラー
//! - 列数不一致はエラー

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;

fn fixture_path(name: &str) -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
        .to_string_lossy()
        .into_owned()
}

// --- stdin で --type なし → エラー ---

#[test]
fn schema_stdin_without_type_fails() {
    let mut cmd = Command::cargo_bin("tabstruct").unwrap();
    cmd.args(["schema", "--stdin"]).write_stdin("a,b\n1,2");
    cmd.assert().failure().stderr(
        predicate::str::contains("--type").or(predicate::str::contains("required")),
    );
}

#[test]
fn convert_stdin_without_type_fails() {
    let mut cmd = Command::cargo_bin("tabstruct").unwrap();
    cmd.args(["convert", "--stdin", "--json"]).write_stdin(r#"{"a":1}"#);
    cmd.assert().failure().stderr(
        predicate::str::contains("--type").or(predicate::str::contains("required")),
    );
}

// --- 不正拡張子 ---

#[test]
fn unsupported_extension_fails() {
    let dir = std::env::temp_dir();
    let path = dir.join("data.txt");
    fs::write(&path, "a,b\n1,2").unwrap();
    let mut cmd = Command::cargo_bin("tabstruct").unwrap();
    cmd.args(["schema", "--file", path.to_str().unwrap()]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Unsupported file extension"));
    fs::remove_file(&path).ok();
}

#[test]
fn convert_unsupported_extension_fails() {
    let dir = std::env::temp_dir();
    let path = dir.join("out.txt");
    fs::write(&path, "a,b\n1,2").unwrap();
    let mut cmd = Command::cargo_bin("tabstruct").unwrap();
    cmd.args(["convert", "--file", path.to_str().unwrap(), "--json"]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Unsupported file extension"));
    fs::remove_file(&path).ok();
}

// --- 拡張子と内容不一致 ---

#[test]
fn json_extension_with_invalid_content_fails() {
    let dir = std::env::temp_dir();
    let path = dir.join("not_json.json");
    fs::write(&path, "this is not json").unwrap();
    let mut cmd = Command::cargo_bin("tabstruct").unwrap();
    cmd.args(["schema", "--file", path.to_str().unwrap()]);
    cmd.assert()
        .failure()
        .stderr(
            predicate::str::contains("could not be parsed")
                .or(predicate::str::contains("JSON parse error")),
        );
    fs::remove_file(&path).ok();
}

#[test]
fn yaml_extension_with_invalid_content_fails() {
    let dir = std::env::temp_dir();
    let path = dir.join("not_yaml.yaml");
    fs::write(&path, "{{{ invalid").unwrap();
    let mut cmd = Command::cargo_bin("tabstruct").unwrap();
    cmd.args(["schema", "--file", path.to_str().unwrap()]);
    cmd.assert()
        .failure()
        .stderr(
            predicate::str::contains("could not be parsed")
                .or(predicate::str::contains("YAML"))
                .or(predicate::str::contains("parse")),
        );
    fs::remove_file(&path).ok();
}

// --- 不正ヘッダ（.. を含む等） ---

#[test]
fn invalid_csv_header_fails() {
    let path = fixture_path("invalid_header_double_dot.csv");
    let mut cmd = Command::cargo_bin("tabstruct").unwrap();
    cmd.args(["schema", "--file", &path]);
    cmd.assert()
        .failure()
        .stderr(
            predicate::str::contains("Invalid CSV header")
                .or(predicate::str::contains("header")),
        );
}

// --- array field の CSV 変換はエラー ---

#[test]
fn array_field_to_csv_fails() {
    let path = fixture_path("array_field.json");
    let mut cmd = Command::cargo_bin("tabstruct").unwrap();
    cmd.args(["convert", "--file", &path, "--csv"]);
    cmd.assert()
        .failure()
        .stderr(
            predicate::str::contains("Array field")
                .and(predicate::str::contains("cannot be converted to CSV")),
        );
}

// --- 列数不一致はエラー ---

#[test]
fn csv_column_count_mismatch_fails() {
    let path = fixture_path("column_mismatch.csv");
    let mut cmd = Command::cargo_bin("tabstruct").unwrap();
    cmd.args(["schema", "--file", &path]);
    cmd.assert()
        .failure()
        .stderr(
            predicate::str::contains("columns")
                .or(predicate::str::contains("column"))
                .or(predicate::str::contains("row")),
        );
}

#[test]
fn convert_csv_column_mismatch_fails() {
    let path = fixture_path("column_mismatch.csv");
    let mut cmd = Command::cargo_bin("tabstruct").unwrap();
    cmd.args(["convert", "--file", &path, "--json"]);
    cmd.assert().failure();
}
