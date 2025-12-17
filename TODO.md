# winh

## Goal

- Windows11 で動く
    - written in Rust
    - this can be compiled on any platform (e.g. WSL)
    - 単一の exe ファイルが生成される (without installer)
- 単一座面だけ持つGUI
- スタートボタンだけが大きくある
    - 押すとボタンは停止ボタンに変わる
    - Windows規定の音声入力から音声入力を受け付ける
    - 2秒間音声入力がなければ停止
        - そこまでの音声入力を mp3 で一時的に保存
        - openai/gpt-4o-transcribe に投げる
        - テキストが返ってくる
            - テキストをGUIに表示
            - さらに、クリップボードにコピー
- 簡易設定
    - 設定画面をモーダルで開く
        - 無音検出の秒数設定 (デフォルト2秒)
        - OpenAI API Key の設定
        - openai model の設定 (デフォルト gpt-4o-transcribe)
    - 以上の設定はローカルに保存

## TODO

### Phase 1: 基本セットアップ ✅ 完了
- [x] GUIフレームワークの選定と導入（egui 0.29を採用）
- [x] 基本的なウィンドウとスタート/停止ボタンのUI実装
- [x] ボタン状態の切り替え機能（⏺ Start / ⏹ Stop）
- [x] Makefileの作成とクロスコンパイル設定
- [x] mingw-w64依存関係の設定
- [x] README.mdの作成

### Phase 2: 音声入力機能
- [ ] Windows音声入力APIの調査と選定（cpal等）
- [ ] 音声入力のキャプチャ機能実装
- [ ] 音声データのバッファリング

### Phase 3: 音声処理
- [ ] 無音検出ロジックの実装（振幅しきい値ベース）
- [ ] 音声データをmp3形式で一時保存する機能
- [ ] 一時ファイルの管理（保存先、削除処理）

### Phase 4: OpenAI API連携
- [ ] HTTP clientの導入（reqwest等）
- [ ] OpenAI Whisper API (gpt-4o-transcribe) への音声ファイル送信機能
- [ ] API レスポンスのパース処理
- [ ] エラーハンドリング（ネットワークエラー、APIエラー等）

### Phase 5: 結果表示とクリップボード
- [ ] 文字起こし結果をGUIに表示
- [ ] クリップボードへのテキストコピー機能（clipboard-win等）

### Phase 6: 設定機能
- [ ] 設定データ構造の定義（無音検出秒数、API Key、モデル名）
- [ ] 設定画面のモーダルUI実装
- [ ] 設定の読み込み・保存機能（JSONまたはTOML形式）
- [ ] 設定ファイルの保存先決定（%APPDATA%等）

### Phase 7: 仕上げ
- [ ] エラー表示UI（API key未設定、ネットワークエラー等）
- [ ] リリースビルドの動作確認
- [ ] Windows11での実機テスト

## 進捗状況

- **Phase 1**: ✅ 完了 (2025-12-17)
  - GUI基盤の構築完了
  - Windows向けビルド環境構築完了
  - 生成物: `winh.exe` (基本UI実装済み)

- **Phase 2-7**: 未着手
  - 次のステップ: 音声入力機能の実装

## @CLAUDE

- このファイルは適宜読み直してチェックして
- あなたも、このファイルを編集できる
- Phase完了時は進捗状況セクションを更新すること
