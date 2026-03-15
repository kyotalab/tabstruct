# tabstruct

CSV / JSON / YAML を相互変換し、データ構造を確認できる CLI ツールです。

## Overview

`tabstruct` は、CSV・JSON・YAML を扱う Rust 製 CLI です。主な用途は、CSV で管理された設定データを JSON や YAML へ変換することです（例: AWS CDK 設定ファイルの生成）。

## Features

- **schema** … 入力の形式・ルート型・レコード数・フィールドと型を表示
- **convert** … CSV ↔ JSON ↔ YAML の相互変換

対応フォーマット: **CSV**, **JSON**, **YAML**

## Installation

```bash
cargo install --path .
```

## Usage

```bash
tabstruct --help
tabstruct schema --help
tabstruct convert --help
```

## Examples

### schema

```bash
tabstruct schema --file sample.csv
```

### convert

```bash
tabstruct convert --file sample.csv --json
tabstruct convert --file sample.csv --yaml
tabstruct convert --file sample.json --csv

# 出力をファイルへ
tabstruct convert --file sample.csv --yaml --output out.yaml
```

### stdin

`--stdin` を使う場合は `--type` が必須です。

```bash
cat sample.csv | tabstruct convert --stdin --type csv --json
```

### サンプルで試す

リポジトリの `examples/` にサンプルファイルがあります。

```bash
tabstruct schema --file examples/sample.csv
tabstruct convert --file examples/sample.csv --json
tabstruct convert --file examples/sample.json --yaml
```

## Limitations

- **CSV**
  - ヘッダ行必須
  - 区切り文字はカンマ固定
  - ネストは `a.b.c` のドット記法で表現
- **JSON/YAML → CSV**
  - 配列フィールドを含む構造は変換不可

## Development

```bash
cargo build
cargo test
```
