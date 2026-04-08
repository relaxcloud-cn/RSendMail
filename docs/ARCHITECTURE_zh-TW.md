# RSendMail 架構說明

[English](ARCHITECTURE.md) | [简体中文](ARCHITECTURE_zh.md) | 繁體中文 | [日本語](ARCHITECTURE_ja.md)

本文檔描述目前 RSendMail 在統一到 Tauri GUI 之後的整體架構。

## 概覽

RSendMail 目前保留兩個使用者入口：

- `rsendmail-cli`：命令列工具
- `rsendmail-tauri`：桌面圖形介面

兩者共用同一個 `rsendmail-core` 業務核心，因此郵件解析、SMTP 發送、重試、附件處理與統計邏輯保持一致。

```text
┌────────────────────────────────────────────────────────────┐
│                         應用入口                           │
├───────────────────────────┬────────────────────────────────┤
│      rsendmail-cli        │        rsendmail-tauri        │
│        Rust CLI           │   Tauri + Vue 桌面 GUI        │
├───────────────────────────┴────────────────────────────────┤
│                      rsendmail-core                        │
│          共用的發送器、設定、統計與匿名化邏輯              │
├────────────────────────────────────────────────────────────┤
│                      rsendmail-i18n                        │
│                  Rust 端共用國際化資源                     │
└────────────────────────────────────────────────────────────┘
```

## 專案結構

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
│       ├── src/                    # Vue 3 + TypeScript 前端
│       ├── src-tauri/              # Rust Tauri 外殼與命令
│       ├── package.json
│       └── vite.config.ts
├── assets/
│   └── screenshots/
└── docs/
    └── ARCHITECTURE_zh-TW.md
```

## 依賴關係

```text
rsendmail-cli ─────► rsendmail-core ─────► rsendmail-i18n

rsendmail-tauri
  ├── 前端層（Vue、vue-i18n、@tauri-apps/api）
  └── src-tauri Rust 外殼 ───────────────► rsendmail-core
```

## 核心元件

### 1. `rsendmail-i18n`

基於 `rust-i18n` 的 Rust 端共用國際化模組。

- 翻譯檔位於 `crates/rsendmail-i18n/locales/`
- 負責 Rust 程式碼的語言偵測
- 提供 `tr()` 與 `tr_with_args()` 介面

### 2. `rsendmail-core`

CLI 與 GUI 共用的郵件發送核心。

- `config.rs`：可序列化的執行設定
- `mailer.rs`：EML 發送、附件發送、重試、SMTP 工作階段處理
- `stats.rs`：計數、耗時、吞吐與報告
- `anonymizer.rs`：郵箱位址匿名化

### 3. `rsendmail-cli`

面向自動化與腳本情境的 Rust 命令列程式。

- 使用 `clap` 解析本地化參數
- 初始化日誌與可選日誌檔輸出
- 在 `Mailer` 之上執行循環與重複發送
- 保持既有 CLI 行為穩定

### 4. `rsendmail-tauri`

桌面 GUI 分成兩層：

- `src/`：Vue 3 + TypeScript + Vite 介面
- `src-tauri/`：Rust Tauri 外殼，向前端暴露命令

前端職責：

- 圖形化收集 SMTP 與發送模式設定
- 透過 Tauri 命令啟動與停止發送
- 監聽 Rust 發出的日誌、進度與統計事件
- 透過 `vue-i18n` 呈現多語言介面

Rust 外殼職責：

- 接收前端傳入的 `Config`
- 直接重用 `rsendmail-core::Mailer`
- 透過 Tauri 事件把執行日誌與狀態推送到 GUI
- 以 `Arc<AtomicBool>` 維護執行狀態

## 資料流

### CLI 流程

```text
CLI 參數
  -> rsendmail-cli
  -> Config
  -> rsendmail-core::Mailer
  -> SMTP / 檔案系統操作
  -> 日誌與統計輸出
```

### GUI 流程

```text
Vue 介面
  -> invoke("start_sending", Config)
  -> Tauri 命令處理器
  -> rsendmail-core::Mailer
  -> 發出日誌 / 進度 / 統計事件
  -> Vue 監聽器更新桌面介面
```

## 關鍵依賴

| 依賴 | 用途 |
|------|------|
| `tokio` | 非同步執行時 |
| `mail-send` | SMTP 客戶端 |
| `mail-parser` | EML 解析 |
| `mail-builder` | 郵件與附件建構 |
| `clap` | CLI 參數解析 |
| `tauri` | 桌面應用外殼 |
| `vue` | GUI 元件執行時 |
| `vite` | 前端建置工具 |
| `vue-i18n` | GUI 國際化 |
| `serde` / `serde_json` | 共用設定序列化 |

## 並發與狀態

- `Arc<AtomicBool>` 控制發送任務的啟動與停止
- `Mutex<Option<AppHandle>>` 讓 Rust 日誌器把訊息轉發到 GUI
- Tokio 任務保證發送期間桌面介面仍保持回應

## 設定模型

`Config` 是 CLI 與 GUI 共用的契約。

- CLI 透過參數解析建立它
- Tauri GUI 從 Vue 前端傳入 Rust 命令
- Serde 保證儲存、載入與互操作穩定

## 維護約束

- 除非明確提出需求，否則不要破壞 CLI 行為
- GUI 相關改動應盡量限制在 `crates/rsendmail-tauri/`
- 郵件發送行為的修改通常應放在 `rsendmail-core`
