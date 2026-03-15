# tabstruct

CSV / JSON / YAML を相互変換し、データ構造を確認できる CLI ツールです。

- **schema**: 入力の構造（形式・ルート型・レコード数・フィールドと型）を表示
- **convert**: CSV ↔ JSON ↔ YAML の変換

## インストール

```bash
cargo install --path .
```

## 最小利用例

### 構造の確認 (schema)

```bash
# ファイルを指定して構造を表示
tabstruct schema --file sample.csv
tabstruct schema --file config.json
tabstruct schema --file config.yaml

# 標準入力（--type 必須）
cat sample.csv | tabstruct schema --stdin --type csv
```

### 形式変換 (convert)

```bash
# CSV → JSON
tabstruct convert --file sample.csv --json

# CSV → YAML
tabstruct convert --file sample.csv --yaml

# JSON → CSV
tabstruct convert --file config.json --csv

# YAML → JSON（または --csv）
tabstruct convert --file config.yaml --json

# 出力をファイルへ
tabstruct convert --file sample.csv --yaml --output out.yaml

# 標準入力
cat sample.json | tabstruct convert --stdin --type json --yaml
```

### サンプルで試す

リポジトリの `examples/` にサンプルファイルがあります。

```bash
tabstruct schema --file examples/sample.csv
tabstruct convert --file examples/sample.csv --json
tabstruct convert --file examples/sample.json --yaml
```

## 仕様メモ

- **入力**: ファイルは拡張子で形式を判定（`.csv`, `.json`, `.yaml`, `.yml`）。標準入力の場合は `--type csv|json|yaml` が必須。
- **CSV**: ヘッダ必須・カンマ区切り。ネストは `settings.interval` のようにドットで表現。
- **JSON/YAML → CSV**: 配列フィールドは非対応。ルートはオブジェクトまたはオブジェクトの配列。

## ヘルプ

```bash
tabstruct --help
tabstruct schema --help
tabstruct convert --help
```
