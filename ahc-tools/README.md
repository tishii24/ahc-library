# ahc-tools

## `expander_rs`

解答コードと `ahc_library` を 1 ファイルに展開します。

```bash
cargo run --bin expander_rs -- \
	--solution_path /path/to/solution \
	--ahc_library_path /path/to/ahc_library
```

- 元の Python 実装に近い単純な文字列ベースの移植
- `src/main.rs` の `mod foo;` を展開
- `ahc_library/src/lib.rs` の `pub mod foo;` と `src/foo/mod.rs` の `pub mod bar;` を展開
- `#[test]` / `#[cfg(test)]` ブロックは元実装と同じ単純ルールで除外
- `use ahc_library::...` は `use crate::ahc_library::...` に変換
- ライブラリ全体に対して `crate::` を `crate::ahc_library::` に置換し、その後マクロ import だけ戻す

`AHC_LIBRARY_PATH` 環境変数からもライブラリパスを受け取れます。`--skip_rustfmt` を付けると整形を省略します。
