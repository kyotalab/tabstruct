# Changelog

## [0.1.0] - 2025-03-15

### Added

- **schema** コマンド: 入力の構造（形式・ルート型・レコード数・フィールドと型）を表示
  - CSV / JSON / YAML に対応
- **convert** コマンド: 形式の相互変換
  - CSV → JSON
  - CSV → YAML
  - JSON → CSV
  - YAML → CSV
  - JSON ↔ YAML
- 標準入力 (`--stdin`) 対応（`--type` 指定時）
- 出力先ファイル指定 (`--output`)

### Limitations

- CSV は区切り文字がカンマ固定
- CSV はヘッダ行必須
- JSON / YAML → CSV では配列フィールドを含む構造は変換不可

### Notes

- CSV のネスト表現はドット記法（例: `a.b.c`）
- スキーマ表示で nullable な型は `?` で表示
- CSV の空セルは null として扱う
