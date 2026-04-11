# TODO.md

## [x] テスト修正 [2026-04-11 完了]

`remove_punctuation` 関数が全角記号（`！`、`？`、`，`、`．`）を処理していなかった。
`src/openai.rs` の match アームに全角記号を追加して修正。

