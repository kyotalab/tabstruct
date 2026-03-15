//! convert コマンドの JSON <-> YAML 相互変換の CLI 経路テスト。

use assert_cmd::Command;
use predicates::prelude::*;

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
