//! convert コマンドの CLI 経路テスト。
//!
//! - JSON <-> YAML 相互変換
//! - JSON/YAML -> CSV（stdout / ファイル出力）
//! - 配列フィールド含み JSON/YAML -> CSV はエラー

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;

#[test]
fn convert_file_json_to_yaml() {
    let dir = std::env::temp_dir();
    let json_path = dir.join("tabstruct_convert_test_in.json");
    let content = r#"{"a":1,"b":"x"}"#;
    std::fs::write(&json_path, content).unwrap();
    let mut cmd = Command::cargo_bin("tabstruct").unwrap();
    cmd.args(["convert", "--file", json_path.to_str().unwrap(), "--yaml"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("a:")
            .and(predicate::str::contains("1"))
            .and(predicate::str::contains("b:"))
            .and(predicate::str::contains("x")));
    std::fs::remove_file(&json_path).ok();
}

#[test]
fn convert_file_yaml_to_json() {
    let dir = std::env::temp_dir();
    let yaml_path = dir.join("tabstruct_convert_test_in.yaml");
    let content = "a: 1\nb: x\n";
    std::fs::write(&yaml_path, content).unwrap();
    let mut cmd = Command::cargo_bin("tabstruct").unwrap();
    cmd.args(["convert", "--file", yaml_path.to_str().unwrap(), "--json"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\"a\"")
            .and(predicate::str::contains("1"))
            .and(predicate::str::contains("\"b\""))
            .and(predicate::str::contains("x")));
    std::fs::remove_file(&yaml_path).ok();
}

#[test]
fn convert_stdin_json_to_yaml() {
    let mut cmd = Command::cargo_bin("tabstruct").unwrap();
    cmd.args(["convert", "--stdin", "--type", "json", "--yaml"])
        .write_stdin(r#"{"x":2}"#);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("x:").and(predicate::str::contains("2")));
}

#[test]
fn convert_stdin_yaml_to_json() {
    let mut cmd = Command::cargo_bin("tabstruct").unwrap();
    cmd.args(["convert", "--stdin", "--type", "yaml", "--json"])
        .write_stdin("x: 2\n");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\"x\"").and(predicate::str::contains("2")));
}

// --- JSON/YAML -> CSV 経路 ---

#[test]
fn convert_file_json_to_csv() {
    let dir = std::env::temp_dir();
    let json_path = dir.join("tabstruct_convert_json_to_csv_in.json");
    let content = r#"{"id":1,"name":"a","nested":{"x":2}}"#;
    std::fs::write(&json_path, content).unwrap();
    let mut cmd = Command::cargo_bin("tabstruct").unwrap();
    cmd.args(["convert", "--file", json_path.to_str().unwrap(), "--csv"]);
    cmd.assert()
        .success()
        .stdout(
            predicate::str::contains("id,name,nested.x")
                .and(predicate::str::contains("1,a,2")),
        );
    std::fs::remove_file(&json_path).ok();
}

#[test]
fn convert_file_yaml_to_csv() {
    let dir = std::env::temp_dir();
    let yaml_path = dir.join("tabstruct_convert_yaml_to_csv_in.yaml");
    let content = "id: 1\nname: a\nnested:\n  x: 2\n";
    std::fs::write(&yaml_path, content).unwrap();
    let mut cmd = Command::cargo_bin("tabstruct").unwrap();
    cmd.args(["convert", "--file", yaml_path.to_str().unwrap(), "--csv"]);
    cmd.assert()
        .success()
        .stdout(
            predicate::str::contains("id,name,nested.x")
                .and(predicate::str::contains("1,a,2")),
        );
    std::fs::remove_file(&yaml_path).ok();
}

#[test]
fn convert_stdin_json_to_csv() {
    let mut cmd = Command::cargo_bin("tabstruct").unwrap();
    cmd.args(["convert", "--stdin", "--type", "json", "--csv"])
        .write_stdin(r#"{"id":1,"name":"b"}"#);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("id,name").and(predicate::str::contains("1,b")));
}

#[test]
fn convert_stdin_yaml_to_csv() {
    let mut cmd = Command::cargo_bin("tabstruct").unwrap();
    cmd.args(["convert", "--stdin", "--type", "yaml", "--csv"])
        .write_stdin("id: 1\nname: b\n");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("id,name").and(predicate::str::contains("1,b")));
}

#[test]
fn convert_file_json_to_csv_output() {
    let dir = std::env::temp_dir();
    let json_path = dir.join("tabstruct_convert_json_to_csv_in2.json");
    let out_path = dir.join("tabstruct_convert_json_to_csv_out.csv");
    let content = r#"{"id":1,"name":"out-test"}"#;
    std::fs::write(&json_path, content).unwrap();
    let mut cmd = Command::cargo_bin("tabstruct").unwrap();
    cmd.args([
        "convert",
        "--file",
        json_path.to_str().unwrap(),
        "--csv",
        "--output",
        out_path.to_str().unwrap(),
    ]);
    cmd.assert().success().stdout(predicate::str::is_empty());
    let out_content = fs::read_to_string(&out_path).unwrap();
    assert!(out_content.contains("id,name"));
    assert!(out_content.contains("1,out-test"));
    std::fs::remove_file(&json_path).ok();
    std::fs::remove_file(&out_path).ok();
}

#[test]
fn convert_file_json_with_array_field_to_csv_fails() {
    let dir = std::env::temp_dir();
    let json_path = dir.join("tabstruct_convert_array_field_in.json");
    let content = r#"{"id":1,"targets":["a","b"]}"#;
    std::fs::write(&json_path, content).unwrap();
    let mut cmd = Command::cargo_bin("tabstruct").unwrap();
    cmd.args(["convert", "--file", json_path.to_str().unwrap(), "--csv"]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Array field")
            .and(predicate::str::contains("cannot be converted to CSV")));
    std::fs::remove_file(&json_path).ok();
}
