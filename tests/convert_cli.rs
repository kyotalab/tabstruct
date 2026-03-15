//! convert コマンドの CLI 結合テスト。
//!
//! 仕様カバー:
//! - CSV -> JSON, CSV -> YAML
//! - JSON -> CSV, YAML -> CSV
//! - JSON -> YAML, YAML -> JSON
//! - stdin 時は --type 必須
//! - file 入力 / --output 出力

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

// --- CSV -> JSON / CSV -> YAML ---

#[test]
fn convert_csv_to_json_file() {
    let path = fixture_path("convert_sample.csv");
    let mut cmd = Command::cargo_bin("tabstruct").unwrap();
    cmd.args(["convert", "--file", &path, "--json"]);
    cmd.assert().success().stdout(
        predicate::str::contains("\"id\"")
            .and(predicate::str::contains("\"name\""))
            .and(predicate::str::contains("\"settings\""))
            .and(predicate::str::contains("1"))
            .and(predicate::str::contains("canary-a"))
            .and(predicate::str::contains("5"))
            .and(predicate::str::contains("https://example.com")),
    );
}

#[test]
fn convert_csv_to_yaml_file() {
    let path = fixture_path("convert_sample.csv");
    let mut cmd = Command::cargo_bin("tabstruct").unwrap();
    cmd.args(["convert", "--file", &path, "--yaml"]);
    cmd.assert().success().stdout(
        predicate::str::contains("id:")
            .and(predicate::str::contains("name:"))
            .and(predicate::str::contains("settings:"))
            .and(predicate::str::contains("interval:"))
            .and(predicate::str::contains("url:"))
            .and(predicate::str::contains("1"))
            .and(predicate::str::contains("canary-a")),
    );
}

#[test]
fn convert_csv_to_json_stdin() {
    let csv = "a,b\n1,2\n3,4\n";
    let mut cmd = Command::cargo_bin("tabstruct").unwrap();
    cmd.args(["convert", "--stdin", "--type", "csv", "--json"])
        .write_stdin(csv);
    cmd.assert().success().stdout(
        predicate::str::contains("\"a\"")
            .and(predicate::str::contains("\"b\""))
            .and(predicate::str::contains("1"))
            .and(predicate::str::contains("2")),
    );
}

#[test]
fn convert_csv_to_yaml_stdin() {
    let csv = "a,b\n1,2\n";
    let mut cmd = Command::cargo_bin("tabstruct").unwrap();
    cmd.args(["convert", "--stdin", "--type", "csv", "--yaml"])
        .write_stdin(csv);
    cmd.assert().success().stdout(
        predicate::str::contains("a:")
            .and(predicate::str::contains("b:"))
            .and(predicate::str::contains("1"))
            .and(predicate::str::contains("2")),
    );
}

// --- JSON -> CSV / YAML -> CSV ---

#[test]
fn convert_json_to_csv_file() {
    let path = fixture_path("valid_json_content.json");
    let mut cmd = Command::cargo_bin("tabstruct").unwrap();
    cmd.args(["convert", "--file", &path, "--csv"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("x").and(predicate::str::contains("1")));
}

#[test]
fn convert_json_to_csv_nested() {
    let json = r#"{"id":1,"name":"a","nested":{"x":2}}"#;
    let mut cmd = Command::cargo_bin("tabstruct").unwrap();
    cmd.args(["convert", "--stdin", "--type", "json", "--csv"])
        .write_stdin(json);
    cmd.assert().success().stdout(
        predicate::str::contains("id,name,nested.x").and(predicate::str::contains("1,a,2")),
    );
}

#[test]
fn convert_yaml_to_csv_file() {
    let dir = std::env::temp_dir();
    let yaml_path = dir.join("convert_yaml_to_csv_in.yaml");
    let content = "id: 1\nname: a\nnested:\n  x: 2\n";
    fs::write(&yaml_path, content).unwrap();
    let mut cmd = Command::cargo_bin("tabstruct").unwrap();
    cmd.args(["convert", "--file", yaml_path.to_str().unwrap(), "--csv"]);
    cmd.assert().success().stdout(
        predicate::str::contains("id,name,nested.x").and(predicate::str::contains("1,a,2")),
    );
    fs::remove_file(&yaml_path).ok();
}

#[test]
fn convert_yaml_to_csv_stdin() {
    let yaml = "id: 1\nname: b\n";
    let mut cmd = Command::cargo_bin("tabstruct").unwrap();
    cmd.args(["convert", "--stdin", "--type", "yaml", "--csv"])
        .write_stdin(yaml);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("id,name").and(predicate::str::contains("1,b")));
}

// --- JSON -> YAML / YAML -> JSON ---

#[test]
fn convert_json_to_yaml_file() {
    let path = fixture_path("valid_json_content.json");
    let mut cmd = Command::cargo_bin("tabstruct").unwrap();
    cmd.args(["convert", "--file", &path, "--yaml"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("x:").and(predicate::str::contains("1")));
}

#[test]
fn convert_yaml_to_json_file() {
    let dir = std::env::temp_dir();
    let yaml_path = dir.join("convert_yaml_to_json_in.yaml");
    let content = "a: 1\nb: x\n";
    fs::write(&yaml_path, content).unwrap();
    let mut cmd = Command::cargo_bin("tabstruct").unwrap();
    cmd.args(["convert", "--file", yaml_path.to_str().unwrap(), "--json"]);
    cmd.assert().success().stdout(
        predicate::str::contains("\"a\"")
            .and(predicate::str::contains("1"))
            .and(predicate::str::contains("\"b\""))
            .and(predicate::str::contains("x")),
    );
    fs::remove_file(&yaml_path).ok();
}

#[test]
fn convert_json_to_yaml_stdin() {
    let mut cmd = Command::cargo_bin("tabstruct").unwrap();
    cmd.args(["convert", "--stdin", "--type", "json", "--yaml"])
        .write_stdin(r#"{"x":2}"#);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("x:").and(predicate::str::contains("2")));
}

#[test]
fn convert_yaml_to_json_stdin() {
    let mut cmd = Command::cargo_bin("tabstruct").unwrap();
    cmd.args(["convert", "--stdin", "--type", "yaml", "--json"])
        .write_stdin("x: 2\n");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\"x\"").and(predicate::str::contains("2")));
}

// --- output 経路: ファイル出力 ---

#[test]
fn convert_file_to_output_path() {
    let path = fixture_path("convert_sample.csv");
    let out = std::env::temp_dir().join("tabstruct_convert_out.json");
    let mut cmd = Command::cargo_bin("tabstruct").unwrap();
    cmd.args([
        "convert",
        "--file",
        &path,
        "--json",
        "--output",
        out.to_str().unwrap(),
    ]);
    cmd.assert().success().stdout(predicate::str::is_empty());
    let content = fs::read_to_string(&out).unwrap();
    assert!(content.contains("\"id\""));
    assert!(content.contains("canary-a"));
    fs::remove_file(&out).ok();
}

#[test]
fn convert_stdin_to_output_path() {
    let out = std::env::temp_dir().join("tabstruct_convert_stdin_out.yaml");
    let mut cmd = Command::cargo_bin("tabstruct").unwrap();
    cmd.args([
        "convert",
        "--stdin",
        "--type",
        "json",
        "--yaml",
        "--output",
        out.to_str().unwrap(),
    ])
    .write_stdin(r#"{"k":1}"#);
    cmd.assert().success().stdout(predicate::str::is_empty());
    let content = fs::read_to_string(&out).unwrap();
    assert!(content.contains("k:") || content.contains("1"));
    fs::remove_file(&out).ok();
}
