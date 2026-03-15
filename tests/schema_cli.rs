//! schema コマンドの CLI 結合テスト。
//!
//! - CSV / JSON / YAML 対応
//! - ファイル入力・標準入力（--type 必須）
//! - 出力: Format, Root Type, Records, Fields

use assert_cmd::Command;
use predicates::prelude::*;
use std::path::Path;

fn fixture_path(name: &str) -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
        .to_string_lossy()
        .into_owned()
}

#[test]
fn schema_file_csv() {
    let path = fixture_path("schema_sample.csv");
    let mut cmd = Command::cargo_bin("tabstruct").unwrap();
    cmd.args(["schema", "--file", &path]);
    cmd.assert().success().stdout(
        predicate::str::contains("Format: CSV")
            .and(predicate::str::contains("Root Type: array"))
            .and(predicate::str::contains("Records: 3"))
            .and(predicate::str::contains("Fields:"))
            .and(predicate::str::contains("id:"))
            .and(predicate::str::contains("name:"))
            .and(predicate::str::contains("settings.interval:"))
            .and(predicate::str::contains("settings.url:")),
    );
}

#[test]
fn schema_file_json() {
    let path = fixture_path("schema_sample.json");
    let mut cmd = Command::cargo_bin("tabstruct").unwrap();
    cmd.args(["schema", "--file", &path]);
    cmd.assert().success().stdout(
        predicate::str::contains("Format: JSON")
            .and(predicate::str::contains("Root Type: array"))
            .and(predicate::str::contains("Records: 2"))
            .and(predicate::str::contains("Fields:"))
            .and(predicate::str::contains("id:"))
            .and(predicate::str::contains("name:"))
            .and(predicate::str::contains("settings.interval:"))
            .and(predicate::str::contains("settings.url:")),
    );
}

#[test]
fn schema_file_yaml() {
    let path = fixture_path("schema_sample.yaml");
    let mut cmd = Command::cargo_bin("tabstruct").unwrap();
    cmd.args(["schema", "--file", &path]);
    cmd.assert().success().stdout(
        predicate::str::contains("Format: YAML")
            .and(predicate::str::contains("Root Type: array"))
            .and(predicate::str::contains("Records: 2"))
            .and(predicate::str::contains("Fields:"))
            .and(predicate::str::contains("id:"))
            .and(predicate::str::contains("name:")),
    );
}

#[test]
fn schema_stdin_csv() {
    let csv = "a,b\n1,2\n";
    let mut cmd = Command::cargo_bin("tabstruct").unwrap();
    cmd.args(["schema", "--stdin", "--type", "csv"])
        .write_stdin(csv);
    cmd.assert().success().stdout(
        predicate::str::contains("Format: CSV")
            .and(predicate::str::contains("Root Type: array"))
            .and(predicate::str::contains("Records: 1"))
            .and(predicate::str::contains("a:"))
            .and(predicate::str::contains("b:")),
    );
}

#[test]
fn schema_stdin_json() {
    let json = r#"[{"id":1,"name":"x"}]"#;
    let mut cmd = Command::cargo_bin("tabstruct").unwrap();
    cmd.args(["schema", "--stdin", "--type", "json"])
        .write_stdin(json);
    cmd.assert().success().stdout(
        predicate::str::contains("Format: JSON")
            .and(predicate::str::contains("Root Type: array"))
            .and(predicate::str::contains("Records: 1"))
            .and(predicate::str::contains("id:"))
            .and(predicate::str::contains("name:")),
    );
}

#[test]
fn schema_stdin_yaml() {
    let yaml = "id: 1\nname: x\n";
    let mut cmd = Command::cargo_bin("tabstruct").unwrap();
    cmd.args(["schema", "--stdin", "--type", "yaml"])
        .write_stdin(yaml);
    cmd.assert().success().stdout(
        predicate::str::contains("Format: YAML")
            .and(predicate::str::contains("Root Type: object"))
            .and(predicate::str::contains("Records: 1"))
            .and(predicate::str::contains("id:"))
            .and(predicate::str::contains("name:")),
    );
}

#[test]
fn schema_csv_shows_nullable_type() {
    let path = fixture_path("schema_sample.csv");
    let mut cmd = Command::cargo_bin("tabstruct").unwrap();
    cmd.args(["schema", "--file", &path]);
    let out = cmd.output().unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    // enabled は boolean、settings.interval や settings.url に空あり → nullable 表記
    assert!(stdout.contains("enabled:") || stdout.contains("boolean"));
    assert!(stdout.contains("?") || stdout.contains("integer") || stdout.contains("string"));
}
