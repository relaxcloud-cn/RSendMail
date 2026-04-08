# RSendMail アーキテクチャ

[English](ARCHITECTURE.md) | [简体中文](ARCHITECTURE_zh.md) | [繁體中文](ARCHITECTURE_zh-TW.md) | 日本語

この文書は、GUI を Tauri に統一した後の RSendMail の現在のアーキテクチャを説明します。

## 概要

RSendMail は現在、2 つの利用者向け入口を持っています。

- `rsendmail-cli`: コマンドラインツール
- `rsendmail-tauri`: デスクトップ GUI

両方とも同じ `rsendmail-core` を共有しており、メール解析、SMTP 送信、再試行、添付処理、統計ロジックを共通化しています。

```text
┌────────────────────────────────────────────────────────────┐
│                        アプリ層                            │
├───────────────────────────┬────────────────────────────────┤
│      rsendmail-cli        │        rsendmail-tauri        │
│        Rust CLI           │   Tauri + Vue デスクトップGUI │
├───────────────────────────┴────────────────────────────────┤
│                      rsendmail-core                        │
│        共有メーラー、設定、統計、匿名化ロジック           │
├────────────────────────────────────────────────────────────┤
│                      rsendmail-i18n                        │
│               Rust 側の共有翻訳リソース                    │
└────────────────────────────────────────────────────────────┘
```

## プロジェクト構成

```text
RSendMail/
├── Cargo.toml
├── crates/
│   ├── rsendmail-i18n/
│   │   ├── locales/
│   │   └── src/lib.rs
│   ├── rsendmail-core/
│   │   └── src/
│   │       ├── anonymizer.rs
│   │       ├── config.rs
│   │       ├── lib.rs
│   │       ├── mailer.rs
│   │       └── stats.rs
│   ├── rsendmail-cli/
│   │   └── src/
│   │       ├── args.rs
│   │       ├── logging.rs
│   │       └── main.rs
│   └── rsendmail-tauri/
│       ├── src/                    # Vue 3 + TypeScript フロントエンド
│       ├── src-tauri/              # Rust Tauri シェルとコマンド
│       ├── package.json
│       └── vite.config.ts
├── assets/
│   └── screenshots/
└── docs/
    └── ARCHITECTURE_ja.md
```

## 依存関係

```text
rsendmail-cli ─────► rsendmail-core ─────► rsendmail-i18n

rsendmail-tauri
  ├── フロントエンド層（Vue、vue-i18n、@tauri-apps/api）
  └── src-tauri Rust シェル ─────────────► rsendmail-core
```

## 主要コンポーネント

### 1. `rsendmail-i18n`

`rust-i18n` を使う Rust 側の共有国際化モジュールです。

- 翻訳ファイルは `crates/rsendmail-i18n/locales/` に配置
- Rust コード向けの言語判定を担当
- `tr()` と `tr_with_args()` を提供

### 2. `rsendmail-core`

CLI と GUI が共有するメール送信エンジンです。

- `config.rs`: シリアライズ可能な実行設定
- `mailer.rs`: EML 送信、添付送信、再試行、SMTP セッション処理
- `stats.rs`: 件数、時間、スループット、レポート
- `anonymizer.rs`: メールアドレス匿名化

### 3. `rsendmail-cli`

自動化やスクリプト用途の Rust コマンドラインアプリです。

- `clap` でローカライズ済み引数を解析
- ログ初期化と任意のログファイル出力
- `Mailer` の上で loop / repeat を実行
- 既存 CLI 挙動を維持

### 4. `rsendmail-tauri`

デスクトップ GUI は 2 層構成です。

- `src/`: Vue 3 + TypeScript + Vite UI
- `src-tauri/`: フロントエンドへコマンドを公開する Rust Tauri シェル

フロントエンドの責務:

- SMTP と送信モード設定を視覚的に収集
- Tauri コマンド経由で送信開始・停止
- Rust からのログ、進捗、統計イベントを購読
- `vue-i18n` で多言語 UI を描画

Rust シェルの責務:

- フロントエンドから `Config` を受け取る
- `rsendmail-core::Mailer` をそのまま再利用
- Tauri イベントで実行ログと状態を GUI へ転送
- `Arc<AtomicBool>` で実行状態を管理

## データフロー

### CLI フロー

```text
CLI 引数
  -> rsendmail-cli
  -> Config
  -> rsendmail-core::Mailer
  -> SMTP / ファイルシステム処理
  -> ログと統計出力
```

### GUI フロー

```text
Vue UI
  -> invoke("start_sending", Config)
  -> Tauri コマンドハンドラ
  -> rsendmail-core::Mailer
  -> ログ / 進捗 / 統計イベントを送出
  -> Vue リスナーがデスクトップ UI を更新
```

## 主な依存関係

| 依存関係 | 役割 |
|----------|------|
| `tokio` | 非同期ランタイム |
| `mail-send` | SMTP クライアント |
| `mail-parser` | EML 解析 |
| `mail-builder` | メールと添付生成 |
| `clap` | CLI 引数解析 |
| `tauri` | デスクトップアプリシェル |
| `vue` | GUI コンポーネントランタイム |
| `vite` | フロントエンドビルドツール |
| `vue-i18n` | GUI 翻訳 |
| `serde` / `serde_json` | 共有設定シリアライズ |

## 並行性と状態

- `Arc<AtomicBool>` で送信処理の開始・停止を制御
- `Mutex<Option<AppHandle>>` により Rust ロガーから GUI へメッセージ転送
- Tokio タスクにより送信中でも GUI 応答性を維持

## 設定モデル

`Config` は CLI と GUI が共有する契約です。

- CLI は引数解析から構築
- Tauri GUI は Vue フロントエンドから Rust コマンドへ送信
- Serde により保存、読込、相互運用を安定化

## 保守方針

- 明示的な要求がない限り CLI 挙動は壊さない
- GUI 関連の変更はできるだけ `crates/rsendmail-tauri/` に閉じ込める
- メール送信ロジックの変更は通常 `rsendmail-core` に実装する
