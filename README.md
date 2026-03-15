# tabstruct

CSV / JSON / YAML を相互変換し、データ構造を確認できる CLI ツールです。

## Overview

`tabstruct` は、CSV・JSON・YAML を扱う Rust 製 CLI です。主な用途は、CSV で管理された設定データを JSON や YAML へ変換することです（例: AWS CDK 設定ファイルの生成）。

## Features

- **schema** … 入力の形式・ルート型・レコード数・フィールドと型を表示
- **convert** … CSV ↔ JSON ↔ YAML の相互変換

対応フォーマット: **CSV**, **JSON**, **YAML**

## Installation

### 前提

- [Rust](https://www.rust-lang.org/)（Cargo 同梱）がインストールされていること。未導入の場合は [rustup](https://rustup.rs/) で導入できます。

### ソースからビルドしてインストール

リポジトリをクローンしたディレクトリで次を実行すると、`~/.cargo/bin/` に `tabstruct` がインストールされます（PATH に `~/.cargo/bin` が含まれている必要があります）。

```bash
git clone https://github.com/kyotalab/tabstruct.git
cd tabstruct
cargo install --path .
```

### crates.io からインストール（未公開）

[crates.io](https://crates.io/) に公開済みの場合は、次のコマンドでインストールできます。

```bash
cargo install tabstruct
```

### Homebrew からインストール

```bash
brew tap kyotalab/tap
brew install tabstruct
```

### GitHub Release のバイナリを使う

[Releases](https://github.com/kyotalab/tabstruct/releases) から、OS・アーキテクチャに合ったアーカイブ（例: `tabstruct-v0.1.0-x86_64-unknown-linux-gnu.tar.gz`）をダウンロードし、展開して得られた `tabstruct`（Windows の場合は `tabstruct.exe`）を PATH の通った場所に置いてください。

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
