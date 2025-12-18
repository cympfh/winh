# winh - TODO

## v0.2.0 (In Progress)

- [x] Transcribed Text エリアはユーザーには編集不可にする (2025-12-18)
    - `TextEdit::multiline().interactive(false)` で編集不可に設定
    - テキストエリアをクリックするとクリップボードにコピー
    - ラベルに "(click to copy)" を表示
- [x] 入力デバイス選択機能 (2025-12-18)
    - Config に `input_device_name` フィールドを追加
    - `audio::get_input_devices()` でデバイス一覧を取得
    - Settings画面に ComboBox でデバイス選択UIを追加
    - 「Windows既定」を先頭に追加（デフォルトデバイスを使用）
    - `AudioRecorder::start_recording_with_device()` で指定デバイスを使用
- [x] ショートカットキー対応（Ctrl+Shift+Hで録音開始）(2025-12-18)
    - `global-hotkey` クレート (v0.6) を使用
    - Ctrl+Shift+H で録音開始（録音中または文字起こし中は動作しない）
    - 他のアプリケーションがフォーカスされている場合でも動作
    - グローバルホットキーマネージャーをアプリケーションに統合
    - 100ms間隔で定期的に再描画をリクエストしてイベント検知を保証
- [x] ショートカットキーを自由に変更できる (2025-12-18)
    - Config に `hotkey` フィールドを追加（デフォルト: "Ctrl+Shift+H"）
    - ホットキー文字列パーサーを実装 (Ctrl, Shift, Alt, Super/Win + A-Z, 0-9, F1-F12)
    - Settings画面にホットキー入力フィールドを追加
    - ホットキー変更時に動的に再登録（旧ホットキー解除→新ホットキー登録）
    - 無効なホットキー形式の場合はエラーメッセージを表示
- [x] 他アプリケーションのテキスト入力フィールドにフォーカスがある場合は読んだテキストを自動入力する (2025-12-18)
    - `enigo` クレート (v0.6.1) を使用
    - `text()` メソッドでUnicode文字列を正しく処理
    - 非ブロッキング実装（バックグラウンドスレッド）
    - エラー発生時は通知のみ
    - Bugfix: v0.2.1では日本語文字が失敗していた → v0.6.1で修正
- [x] メイン画面に２つチェックボックスを追加 (2025-12-18)
    - [x] クリップボードに自動コピーする (default: ON)
    - [x] 他アプリに自動入力する (default: ON)
    - Settings画面の "Output Options" セクションに配置
    - 両方のオプションは独立して動作可能
- [ ] Bugfix: 一部マイクが使えない
    - Case 1:
    ```
    Recording started
    Using input device: マイク (UGREEN Camera Audio)
    Mono config not supported (Failed to build input stream: The requested stream configuration is not supported by the device.), falling back to default config
    Sample rate: 48000Hz, Channels: 2 (using default), Format: F32
    Failed to start recording: Failed to build input stream: A backend-specific error has occurred: 0x88890008
    ```
    - Case 2:
    ```
    Recording started
    Using input device: CABLE-A Output (VB-Audio Cable A)
    Mono config not supported (Failed to build input stream: The requested stream configuration is not supported by the device.), falling back to default config
    Sample rate: 44100Hz, Channels: 2 (using default), Format: F32
    Failed to start recording: Failed to build input stream: A backend-specific error has occurred: 0x8889000A
    ```

## v0.1.0 (Released 2025-12-18)

### Phase 1: 基本セットアップ ✅ 完了
- [x] GUIフレームワークの選定と導入（egui 0.29を採用）
- [x] 基本的なウィンドウとスタート/停止ボタンのUI実装
- [x] ボタン状態の切り替え機能（⏺ Start / ⏹ Stop）
- [x] Makefileの作成とクロスコンパイル設定
- [x] mingw-w64依存関係の設定
- [x] README.mdの作成

### Phase 2: 音声入力機能 ✅ 完了
- [x] Windows音声入力APIの調査と選定（cpal 0.15を採用）
- [x] 音声入力のキャプチャ機能実装（AudioRecorderモジュール）
- [x] 音声データのバッファリング（Arc<Mutex<Vec<f32>>>で実装）
- [x] GUIとの統合（リアルタイム録音情報表示）

### Phase 3: 音声処理 ✅ 完了
- [x] 無音検出ロジックの実装（振幅しきい値ベース、デフォルト0.01）
- [x] 音声データをWAV形式で一時保存する機能（houndクレート使用）
- [x] 一時ファイルの管理（tempfileクレート使用、自動命名）
- [x] 自動無音検出による録音停止機能（デフォルト2秒）
- [x] リアルタイム無音時間表示

### Phase 4: OpenAI API連携 ✅ 完了
- [x] HTTP clientの導入（reqwest 0.12、blocking + multipart機能）
- [x] OpenAI Whisper API への音声ファイル送信機能（multipart/form-data）
- [x] API レスポンスのパース処理（serde_json使用）
- [x] エラーハンドリング（ネットワークエラー、APIエラー、ファイルエラー、パースエラー）
- [x] バックグラウンドスレッドでの文字起こし処理
- [x] GUIへの文字起こし結果表示
- [x] 環境変数OPENAI_API_KEYからAPIキー読み込み

### Phase 5: 結果表示とクリップボード ✅ 完了
- [x] 文字起こし結果をGUIに表示（Phase 4で実装済み）
- [x] クリップボードへのテキストコピー機能（arboard 3.3使用）
- [x] コピー成功・失敗のステータス表示

### Phase 6: 設定機能 ✅ 完了
- [x] 設定データ構造の定義（無音検出秒数、API Key、モデル名、無音しきい値）
- [x] 設定画面のモーダルUI実装（⚙ Settingsボタン）
- [x] 設定の読み込み・保存機能（JSON形式）
- [x] 設定ファイルの保存先決定（dirs crateでクロスプラットフォーム対応）
- [x] コマンドライン引数でのAPI Key指定（OPENAI_API_KEY=...）
- [x] 設定のリアルタイム適用
- [x] モデルのデフォルトは gpt-4o-transcribe に

### Phase 6.5 ✅ 完了

- [x] src/audio.rs
    - 録音開始すぐは無音検出しないように修正 (3sec 余裕を持たせる)
    - save_audio_to_wav では **先頭から** 無音部分を削除 (trim)
        - 0.2秒程度だけ無音を残す

### Phase 7: 仕上げ ✅ 完了
- [x] エラー表示UI（API key未設定、ネットワークエラー等）
- [x] リリースビルドの動作確認
- [x] Windows11での実機テスト
- [x] ドキュメント整備（README更新、使用方法記載）
- [x] .github/workflows
    - Rust のCI設定
    - Windows向けバイナリリリースの自動化
- [x] コードのリファクタリング
    - rustfmt
    - Warning の解消

## 進捗状況

- **Phase 1**: ✅ 完了 (2025-12-17)
  - GUI基盤の構築完了
  - Windows向けビルド環境構築完了
  - 生成物: `winh.exe` (基本UI実装済み)

- **Phase 2**: ✅ 完了 (2025-12-17)
  - cpalを使った音声入力機能実装完了
  - リアルタイム録音＆バッファリング機能
  - 録音時間・サンプル数の表示機能
  - 生成物: `winh.exe` (音声入力機能実装済み)

- **Phase 3**: ✅ 完了 (2025-12-17)
  - 無音検出ロジックの実装完了
  - 自動停止機能（2秒無音で停止）
  - WAV形式での音声保存機能
  - 一時ファイル管理システム
  - 生成物: `winh.exe` (自動録音停止＆保存機能実装済み)

- **Phase 4**: ✅ 完了 (2025-12-17)
  - OpenAI Whisper API連携完了
  - バックグラウンドスレッドでの文字起こし
  - APIエラーハンドリング
  - 文字起こし結果のGUI表示
  - 生成物: `winh.exe` (完全な音声文字起こし機能実装済み)

- **Phase 5**: ✅ 完了 (2025-12-17)
  - 文字起こし結果のGUI表示完了
  - クリップボードへの自動コピー機能
  - エラーハンドリング付きコピー処理
  - 生成物: `winh.exe` (クリップボード機能実装済み)

- **Phase 6**: ✅ 完了 (2025-12-17)
  - 設定機能完了（設定画面UI）
  - JSON形式での設定保存/読み込み
  - コマンドライン引数でのAPI Key指定
  - クロスプラットフォーム対応の設定ファイル保存先
  - 生成物: `winh.exe` (設定機能実装済み)

- **Phase 6.5**: ✅ 完了 (2025-12-17)
  - 録音開始直後の無音検出を防ぐ3秒間の猶予期間を追加
  - 先頭の無音部分を削除する機能を実装（0.2秒のみ保持）
  - API送信前のオーディオファイルサイズ削減と品質向上
  - 生成物: `winh.exe` (改善された音声処理機能)

- **Phase 7**: ✅ 完了 (2025-12-17)
  - コードのリファクタリング（rustfmt、警告解消）
  - エラー表示UIの実装（API key未設定警告、ネットワークエラー表示）
  - リリースビルドの動作確認（30MB実行ファイル生成）
  - READMEの大幅更新（詳細な使用方法、技術スタック）
  - GitHub Actions CI/CDワークフローの実装
  - 生成物: `winh.exe` (完成版、リリース準備完了)

## @CLAUDE

- TODO.md
    - このファイルは適宜読み直してチェックすること
    - あなたも、このファイルを編集できる
    - Phase 完了時は進捗状況セクションを更新すること
- Rust
    - 1.92.0
    - rustfmt でコードを整形すること
    - cargo test でユニットテストを実行すること
