# winh

Windows 11向けの音声入力・文字起こしGUIアプリケーション

## 概要

winhは、音声入力を簡単に文字起こしできるシンプルなデスクトップアプリケーションです。OpenAI Whisper APIを使用して、高精度な音声認識を提供します。

### 主な機能

- **ワンクリック録音**: 大きなスタート/停止ボタンで簡単に音声入力開始
- **自動無音検出**: 設定した秒数の無音を検出すると自動的に録音停止
- **高精度文字起こし**: OpenAI Whisper API (gpt-4o-transcribe) による文字起こし
- **クリップボード連携**: 文字起こし結果を自動的にクリップボードにコピー
- **軽量**: 単一のexeファイルで動作（インストーラー不要）

## セットアップ

### 前提条件

- Rust (最新の安定版)
- mingw-w64ツールチェーン（WSLでビルドする場合）

### WSL/Linuxでのビルド環境構築

1. **依存パッケージのインストール**
   ```bash
   sudo apt update
   sudo apt install -y mingw-w64
   ```

2. **Rustターゲットの追加**
   ```bash
   rustup target add x86_64-pc-windows-gnu
   ```

## ビルド方法

### Makefileを使用する場合（推奨）

```bash
# ヘルプを表示
make help

# 依存関係のインストール（初回のみ）
make install-deps

# デバッグビルド
make build-windows

# リリースビルド（最適化版）
make build-windows-release
```

### 手動でビルドする場合

```bash
# デバッグビルド
cargo build --target x86_64-pc-windows-gnu

# リリースビルド
cargo build --release --target x86_64-pc-windows-gnu
```

### 生成されるファイル

- デバッグ版: `target/x86_64-pc-windows-gnu/debug/winh.exe`
- リリース版: `target/x86_64-pc-windows-gnu/release/winh.exe`

## 使い方

1. **初回起動時の設定**
   - アプリケーションを起動
   - 設定ボタンから OpenAI API キーを設定
   - 無音検出の秒数を調整（デフォルト: 2秒）

2. **音声入力**
   - スタートボタンをクリックして録音開始
   - 話し終えたら、設定した秒数の無音で自動停止
   - または手動で停止ボタンをクリック

3. **文字起こし結果**
   - 画面に文字起こし結果が表示されます
   - 自動的にクリップボードにコピーされます

## 開発状況

現在Phase 1が完了しています。詳細は [TODO.md](TODO.md) を参照してください。

### 完了済み
- ✅ GUIフレームワーク（egui）の導入
- ✅ 基本的なウィンドウとスタート/停止ボタンUI
- ✅ ボタン状態の切り替え機能
- ✅ Windows向けクロスコンパイル環境

### 次のステップ
- 音声入力機能の実装
- 無音検出ロジック
- OpenAI API連携

## 技術スタック

- **言語**: Rust
- **GUIフレームワーク**: egui + eframe
- **クロスコンパイル**: mingw-w64

## ライセンス

未定

## 開発者向け情報

詳細な開発ガイドは [CLAUDE.md](CLAUDE.md) を参照してください。
