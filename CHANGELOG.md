# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0] - 2026-04-22

### Added
- x.ai STT API による音声書き起こし対応
- xAI API Key の設定項目を追加（設定画面の先頭に配置）

### Changed
- 音声書き起こしバックエンドを OpenAI Whisper から x.ai STT API に完全移行
- `src/openai.rs` を `src/text_to_speech.rs` にリネーム（`TextToSpeechClient` クラス）
- CLI 引数を `--api-key=` / `OPENAI_API_KEY=` から `--xai-api-key=` / `XAI_API_KEY=` に変更

### Removed
- OpenAI API Key、Model、Custom Prompt の設定項目を削除
- `src/openai.rs` および `OpenAIClient` クラスを削除

(git commit: 7169ed5)

## [0.2.1] - 2026-04-11

### Fixed
- Eliza クライアントに 60 秒のタイムアウトを追加（無限待機を防止）
- `remove_punctuation` が全角記号（`！`・`？`・`，`・`．`）を除去しない問題を修正

(git commit: 3846e59)

## [0.2.0] - 2026-03-03

### Added
- 書き起こしテキストエリア（クリックでクリップボードコピー）
- 入力デバイス選択
- グローバルホットキー対応（デフォルト: Ctrl+Shift+H）
- ホットキーのカスタマイズ設定
- カスタムプロンプト設定
- 自動入力機能（書き起こし結果を自動でキーボード入力）
- VRChat OSC チャットボックス連携（送信機能）
- VRChat ミュートトリガーで録音開始
- Eliza エージェントサーバー連携（OSC GestureRight トリガー）
- Eliza レスポンスを VRChat チャットボックスへ非同期送信

### Changed
- GUI レイアウトの改善・整理
- 自動入力と VRChat 送信を排他選択に変更
- 書き起こし結果から句読点を除去するように変更

### Fixed
- サイレンスプログレスバーの方向を修正（逆向きだった）
- VRChat OSC ポート番号を 9091 → 9000 に修正

(git commit: 06e7c7b)

## [0.1.0] - 2025-12-18

### Added
- 単一ウィンドウGUIアプリケーション（egui/eframe）
- Windows デフォルト音声入力によるボイス録音（モノラル強制）
- 無音検出（設定可能な閾値・継続時間）
- 無音検出のグレース期間（録音開始から3秒）
- 先頭無音トリミング
- OpenAI Whisper API による音声書き起こし
- クリップボードへの自動コピー
- 設定モーダル（APIキー、モデル選択、無音設定）
- ローカル設定の永続化（JSON形式）
- 日本語フォント対応（Noto Sans JP）
- バックグラウンド書き起こし（ノンブロッキングUI）
- リアルタイム音量レベルインジケーター
- サイレンスプログレスインジケーター
