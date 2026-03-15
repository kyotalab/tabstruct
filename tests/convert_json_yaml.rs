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
    cmd.assert().success().stdout(
        predicate::str::contains("a:")
            .and(predicate::str::contains("1"))
            .and(predicate::str::contains("b:"))
            .and(predicate::str::contains("x")),
    );
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
    cmd.assert().success().stdout(
        predicate::str::contains("\"a\"")
            .and(predicate::str::contains("1"))
            .and(predicate::str::contains("\"b\""))
            .and(predicate::str::contains("x")),
    );
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
    cmd.assert().success().stdout(
        predicate::str::contains("id,name,nested.x").and(predicate::str::contains("1,a,2")),
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
    cmd.assert().success().stdout(
        predicate::str::contains("id,name,nested.x").and(predicate::str::contains("1,a,2")),
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
    cmd.assert().failure().stderr(
        predicate::str::contains("Array field")
            .and(predicate::str::contains("cannot be converted to CSV")),
    );
    std::fs::remove_file(&json_path).ok();
}

// --- CSV -> JSON / CSV -> YAML 経路（L_CSV->JSON・YAML_convert経路）---

#[test]
fn convert_file_csv_to_json() {
    let dir = std::env::temp_dir();
    let csv_path = dir.join("tabstruct_convert_csv_to_json_in.csv");
    let content = "id,name,settings.interval,settings.url\n1,canary-a,5,https://example.com\n2,canary-b,10,https://example.org\n";
    std::fs::write(&csv_path, content).unwrap();
    let mut cmd = Command::cargo_bin("tabstruct").unwrap();
    cmd.args(["convert", "--file", csv_path.to_str().unwrap(), "--json"]);
    cmd.assert().success().stdout(
        predicate::str::contains("\"id\"")
            .and(predicate::str::contains("\"name\""))
            .and(predicate::str::contains("\"settings\""))
            .and(predicate::str::contains("\"interval\""))
            .and(predicate::str::contains("\"url\""))
            .and(predicate::str::contains("1"))
            .and(predicate::str::contains("canary-a"))
            .and(predicate::str::contains("5"))
            .and(predicate::str::contains("https://example.com"))
            .and(predicate::str::contains("2"))
            .and(predicate::str::contains("canary-b")),
    );
    std::fs::remove_file(&csv_path).ok();
}

#[test]
fn convert_file_csv_to_yaml() {
    let dir = std::env::temp_dir();
    let csv_path = dir.join("tabstruct_convert_csv_to_yaml_in.csv");
    let content = "id,name,settings.interval,settings.url\n1,canary-a,5,https://example.com\n2,canary-b,10,https://example.org\n";
    std::fs::write(&csv_path, content).unwrap();
    let mut cmd = Command::cargo_bin("tabstruct").unwrap();
    cmd.args(["convert", "--file", csv_path.to_str().unwrap(), "--yaml"]);
    cmd.assert().success().stdout(
        predicate::str::contains("id:")
            .and(predicate::str::contains("name:"))
            .and(predicate::str::contains("settings:"))
            .and(predicate::str::contains("interval:"))
            .and(predicate::str::contains("url:"))
            .and(predicate::str::contains("1"))
            .and(predicate::str::contains("canary-a"))
            .and(predicate::str::contains("5"))
            .and(predicate::str::contains("https://example.com")),
    );
    std::fs::remove_file(&csv_path).ok();
}

#[test]
fn convert_stdin_csv_to_json() {
    let csv = "a,b\n1,2\n3,4\n";
    let mut cmd = Command::cargo_bin("tabstruct").unwrap();
    cmd.args(["convert", "--stdin", "--type", "csv", "--json"])
        .write_stdin(csv);
    cmd.assert().success().stdout(
        predicate::str::contains("\"a\"")
            .and(predicate::str::contains("\"b\""))
            .and(predicate::str::contains("1"))
            .and(predicate::str::contains("2"))
            .and(predicate::str::contains("3"))
            .and(predicate::str::contains("4")),
    );
}

#[test]
fn convert_stdin_csv_to_yaml() {
    let csv = "a,b\n1,2\n3,4\n";
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

/// null含み: 空セルは JSON で null になる。
#[test]
fn convert_csv_to_json_with_nulls() {
    let csv = "id,name,opt\n1,alice,\n2,,value\n";
    let mut cmd = Command::cargo_bin("tabstruct").unwrap();
    cmd.args(["convert", "--stdin", "--type", "csv", "--json"])
        .write_stdin(csv);
    let out = cmd.output().unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let arr = parsed.as_array().unwrap();
    assert_eq!(arr.len(), 2);
    // row0: id=1, name=alice, opt=null
    assert_eq!(arr[0].get("opt"), Some(&serde_json::Value::Null));
    // row1: id=2, name=null, opt=value
    assert!(arr[1].get("name").map_or(false, |v| v.is_null()));
}

/// nested object: a.b.c ヘッダからネストオブジェクトが復元され JSON に出力される。
#[test]
fn convert_csv_to_json_nested_object() {
    let csv =
        "id,settings.interval,settings.url\n1,5,https://example.com\n2,10,https://example.org\n";
    let mut cmd = Command::cargo_bin("tabstruct").unwrap();
    cmd.args(["convert", "--stdin", "--type", "csv", "--json"])
        .write_stdin(csv);
    let out = cmd.output().unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let arr = parsed.as_array().unwrap();
    assert_eq!(arr.len(), 2);
    let first = arr[0].as_object().unwrap();
    assert_eq!(first.get("id"), Some(&serde_json::json!(1)));
    let settings = first.get("settings").and_then(|v| v.as_object()).unwrap();
    assert_eq!(settings.get("interval"), Some(&serde_json::json!(5)));
    assert_eq!(
        settings.get("url").and_then(|v| v.as_str()),
        Some("https://example.com")
    );
}
