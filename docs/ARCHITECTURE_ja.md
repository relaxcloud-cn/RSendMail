# RSendMail アーキテクチャ

[English](ARCHITECTURE.md) | [简体中文](ARCHITECTURE_zh.md) | [繁體中文](ARCHITECTURE_zh-TW.md) | 日本語

本ドキュメントでは、高性能な大量メール送信ツールである RSendMail のアーキテクチャと設計について説明します。

## 概要

RSendMail は、SMTP 経由でメールのテストと大量送信を行うための Rust ベースのアプリケーションです。CLI と GUI の両方のインターフェースを提供し、共通のコアライブラリを共有しています。

```
┌─────────────────────────────────────────────────────────┐
│                   アプリケーション層                      │
├──────────────────────┬──────────────────────────────────┤
│    rsendmail-cli     │         rsendmail-gui            │
│   (コマンドライン)    │        (Slint GUI)               │
├──────────────────────┴──────────────────────────────────┤
│                   rsendmail-core                         │
│                (メール送信エンジン)                       │
├─────────────────────────────────────────────────────────┤
│                   rsendmail-i18n                         │
│                   (国際化対応)                            │
└─────────────────────────────────────────────────────────┘
```

## プロジェクト構成

```
RSendMail/
├── Cargo.toml                      # ワークスペース設定
├── crates/
│   ├── rsendmail-i18n/             # 国際化サポート
│   │   ├── src/lib.rs              # Language 列挙型、tr() 関数
│   │   └── locales/                # YAML 翻訳ファイル
│   │       ├── en-US.yml           # 英語 (デフォルト)
│   │       ├── zh-CN.yml           # 簡体字中国語
│   │       ├── zh-TW.yml           # 繁体字中国語
│   │       └── ja-JP.yml           # 日本語
│   │
│   ├── rsendmail-core/             # コアライブラリ
│   │   └── src/
│   │       ├── lib.rs              # ライブラリエクスポート
│   │       ├── config.rs           # 設定構造体
│   │       ├── mailer.rs           # メール送信エンジン (~1800 行)
│   │       ├── stats.rs            # 統計情報収集
│   │       └── anonymizer.rs       # メールアドレス匿名化
│   │
│   ├── rsendmail-cli/              # CLI アプリケーション
│   │   └── src/
│   │       ├── main.rs             # エントリポイント、ループ制御
│   │       ├── args.rs             # CLI 引数解析 (clap builder)
│   │       └── logging.rs          # ログ初期化
│   │
│   └── rsendmail-gui/              # GUI アプリケーション
│       ├── src/
│       │   ├── main.rs             # GUI エントリポイント
│       │   └── i18n.rs             # GUI 専用 i18n
│       ├── ui/
│       │   └── app.slint           # UI 定義
│       └── fonts/                  # カスタムフォント
│
├── assets/
│   └── screenshots/                # GUI スクリーンショット
│
└── docs/
    └── ARCHITECTURE.md             # 本ドキュメント
```

## Crate 依存関係

```
rsendmail-cli ──┬──► rsendmail-core ──► rsendmail-i18n
                │
                └──► rsendmail-i18n

rsendmail-gui ──┬──► rsendmail-core ──► rsendmail-i18n
                │
                └──► (GUI は独自の HashMap 方式で i18n を実装)
```

## コアコンポーネント

### 1. rsendmail-i18n

`rust-i18n` ライブラリを使用した共有国際化モジュール。

**機能：**
- 4 言語対応：英語、簡体字中国語、繁体字中国語、日本語
- 環境変数とシステムロケールからの言語検出
- 翻訳関数：`tr(key)` と `tr_with_args(key, args)`
- YAML ベースの翻訳ファイル（各言語約 250 キー）

**言語検出の優先順位：**
1. `--lang` CLI 引数
2. `RSENDMAIL_LANG` 環境変数
3. `LANG` / `LC_ALL` 環境変数
4. macOS `AppleLocale`（macOS のみ）
5. デフォルトは英語

### 2. rsendmail-core

CLI と GUI で共有されるコアメール送信エンジン。

**モジュール：**

#### config.rs
- `Config` 構造体（30 以上の設定オプション）
- Serde シリアライズ対応（保存/読み込み用）
- すべてのオプションフィールドにデフォルト値
- `ProcessMode` 列挙型（Auto / Fixed）

#### mailer.rs (~1800 行)
3 つの動作モードをサポートする主要なメール送信ロジック：

1. **EML モード** (`--dir`)
   - ディレクトリから EML ファイルを読み込み
   - 単一の SMTP セッションでの一括送信をサポート
   - マルチプロセス並列送信

2. **添付ファイルモード** (`--attachment`)
   - 単一ファイルをメール添付として送信
   - MIME タイプの自動検出
   - 件名/本文のテンプレートサポート

3. **添付ファイルディレクトリモード** (`--attachment-dir`)
   - ディレクトリ内の各ファイルを個別メールとして送信
   - 単一添付ファイルモードと同じテンプレートサポート

**接続処理：**
- プレーンテキスト接続（ポート 25）
- STARTTLS（ポート 587）
- 暗黙的 TLS（ポート 465）
- SMTP 認証（ユーザー名/パスワード）
- 接続タイムアウトとリトライロジック
- 接続問題の検出（421 エラー、パイプ破損）

#### stats.rs
- `Stats` 構造体で追跡：
  - メール数（合計、成功、失敗）
  - 解析/送信時間
  - エラー分類とファイルリスト
  - QPS（1 秒あたりのクエリ数）計算
- フォーマット出力用の `Display` trait 実装

#### anonymizer.rs
- メールアドレスをランダム文字列に置換
- 一貫性を維持（同じメール → 同じ置換結果）
- HashMap でキャッシュ

### 3. rsendmail-cli

コマンドラインインターフェースアプリケーション。

**機能：**
- 30 以上のコマンドラインオプション
- ローカライズされた `--help` 出力
- グレースフルシャットダウン（Ctrl+C 処理）
- ループと繰り返しモード
- オプションのログファイル出力
- 失敗メールの保存

**アーキテクチャ：**
- 実行時 i18n のため clap builder パターン（derive ではなく）を使用
- CLI 解析前の言語検出
- Tokio 非同期ランタイム

### 4. rsendmail-gui

Slint フレームワークを使用したグラフィカルユーザーインターフェース。

**機能：**
- ビジュアル SMTP 設定
- 3 つの送信モード（モード専用 UI 付き）
- リアルタイムの進捗と統計
- ログの表示とエクスポート
- 設定の保存/読み込み（JSON）
- 言語切り替え
- デュアル出力用カスタムロガー（ターミナル + GUI）

**UI コンポーネント：**
- タブ付きメインウィンドウ
- SMTP サーバー設定パネル
- 送信モードセレクター
- 詳細オプションパネル
- 統計情報表示
- ログビューア

## データフロー

### CLI フロー
```
main.rs
  │
  ├─► detect_language() ──► set_language()
  │
  ├─► parse_args() ──► Config
  │
  ├─► init_logging()
  │
  ├─► Mailer::new(config)
  │
  └─► ループ:
        │
        ├─► mailer.send_all_with_cancel(running)
        │     │
        │     ├─► EML モード: collect_email_files() → send_fixed_mode()
        │     ├─► 添付ファイルモード: send_attachment_with_cancel()
        │     └─► 添付ファイルディレクトリモード: send_attachment_dir_with_cancel()
        │
        ├─► Stats を累積
        │
        └─► 次のイテレーションを待機（ループ/繰り返しモードの場合）
```

### GUI フロー
```
main.rs
  │
  ├─► init_logger() (GuiLogger)
  │
  ├─► AppWindow::new()
  │
  ├─► setup_i18n()
  │
  ├─► setup_callbacks()
  │     │
  │     ├─► on_start_send() ──► 非同期タスクを起動
  │     │     │
  │     │     └─► Mailer::send_all_with_cancel()
  │     │           │
  │     │           └─► mpsc チャネル経由でイベント送信
  │     │
  │     ├─► on_stop_send() ──► running = false を設定
  │     │
  │     ├─► on_browse_*() ──► ファイルダイアログ
  │     │
  │     └─► on_save/load_config() ──► JSON シリアライズ/デシリアライズ
  │
  └─► app.run()
```

## 主要な依存関係

| 依存関係 | 用途 |
|----------|------|
| tokio | 非同期ランタイム |
| mail-send | SMTP クライアント |
| mail-parser | EML ファイル解析 |
| mail-builder | メール構築 |
| clap | CLI 引数解析 |
| slint | GUI フレームワーク |
| rust-i18n | 国際化 |
| serde | 設定のシリアライズ |
| walkdir | ディレクトリ走査 |
| infer | MIME タイプ検出 |

## エラー処理

- `anyhow::Result` でアプリケーションレベルのエラー
- `Stats.increment_error()` で各メールのエラー追跡
- タイプ別エラー分類（接続、認証、送信、解析）
- 後の分析用に失敗メールファイルを保存

## スレッドセーフティ

- `Arc<AtomicBool>` でグレースフルシャットダウンシグナル
- `Arc<Mutex<...>>` で GUI ロガーの共有状態
- Tokio チャネルで GUI イベント通信
- マルチプロセスモードでプロセスごとの統計情報

## 設定

`Config` 構造体のサポート：
- コード内でのフィールド直接アクセス
- GUI 保存/読み込み用の JSON シリアライズ
- CLI 引数解析
- すべてのオプションフィールドにデフォルト値
