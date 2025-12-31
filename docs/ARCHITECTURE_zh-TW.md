# RSendMail 架構設計

[English](ARCHITECTURE.md) | [简体中文](ARCHITECTURE_zh.md) | 繁體中文 | [日本語](ARCHITECTURE_ja.md)

本文件描述 RSendMail 的架構設計，這是一個高效能的批量郵件發送工具。

## 概述

RSendMail 是一個基於 Rust 的應用程式，用於透過 SMTP 測試和發送批量郵件。它提供 CLI 和 GUI 兩種介面，共用一個核心函式庫。

```
┌─────────────────────────────────────────────────────────┐
│                      應用層                              │
├──────────────────────┬──────────────────────────────────┤
│    rsendmail-cli     │         rsendmail-gui            │
│     (命令列工具)      │        (Slint 圖形介面)           │
├──────────────────────┴──────────────────────────────────┤
│                   rsendmail-core                         │
│                  (郵件發送引擎)                           │
├─────────────────────────────────────────────────────────┤
│                   rsendmail-i18n                         │
│                   (國際化支援)                            │
└─────────────────────────────────────────────────────────┘
```

## 專案結構

```
RSendMail/
├── Cargo.toml                      # 工作空間設定
├── crates/
│   ├── rsendmail-i18n/             # 國際化支援
│   │   ├── src/lib.rs              # Language 列舉，tr() 函式
│   │   └── locales/                # YAML 翻譯檔案
│   │       ├── en-US.yml           # 英文 (預設)
│   │       ├── zh-CN.yml           # 簡體中文
│   │       ├── zh-TW.yml           # 繁體中文
│   │       └── ja-JP.yml           # 日文
│   │
│   ├── rsendmail-core/             # 核心函式庫
│   │   └── src/
│   │       ├── lib.rs              # 函式庫匯出
│   │       ├── config.rs           # 設定結構體
│   │       ├── mailer.rs           # 郵件發送引擎 (~1800 行)
│   │       ├── stats.rs            # 統計資訊收集
│   │       └── anonymizer.rs       # 郵箱匿名化
│   │
│   ├── rsendmail-cli/              # CLI 應用程式
│   │   └── src/
│   │       ├── main.rs             # 進入點，迴圈控制
│   │       ├── args.rs             # CLI 參數解析 (clap builder)
│   │       └── logging.rs          # 日誌初始化
│   │
│   └── rsendmail-gui/              # GUI 應用程式
│       ├── src/
│       │   ├── main.rs             # GUI 進入點
│       │   └── i18n.rs             # GUI 專用 i18n
│       ├── ui/
│       │   └── app.slint           # UI 定義
│       └── fonts/                  # 自訂字型
│
├── assets/
│   └── screenshots/                # GUI 螢幕截圖
│
└── docs/
    └── ARCHITECTURE.md             # 本文件
```

## Crate 相依性

```
rsendmail-cli ──┬──► rsendmail-core ──► rsendmail-i18n
                │
                └──► rsendmail-i18n

rsendmail-gui ──┬──► rsendmail-core ──► rsendmail-i18n
                │
                └──► (GUI 使用獨立的 HashMap 方式實作 i18n)
```

## 核心元件

### 1. rsendmail-i18n

使用 `rust-i18n` 函式庫的共用國際化模組。

**功能：**
- 支援 4 種語言：英文、簡體中文、繁體中文、日文
- 從環境變數和系統區域設定偵測語言
- 翻譯函式：`tr(key)` 和 `tr_with_args(key, args)`
- 基於 YAML 的翻譯檔案（每種語言約 250 個鍵值）

**語言偵測優先順序：**
1. `--lang` CLI 參數
2. `RSENDMAIL_LANG` 環境變數
3. `LANG` / `LC_ALL` 環境變數
4. macOS `AppleLocale`（僅 macOS）
5. 預設使用英文

### 2. rsendmail-core

核心郵件發送引擎，CLI 和 GUI 共用。

**模組：**

#### config.rs
- `Config` 結構體，包含 30+ 設定選項
- 支援 Serde 序列化（用於儲存/載入）
- 所有可選欄位都有預設值
- `ProcessMode` 列舉（Auto / Fixed）

#### mailer.rs (~1800 行)
主要的郵件發送邏輯，支援三種操作模式：

1. **EML 模式** (`--dir`)
   - 從目錄讀取 EML 檔案
   - 支援單一 SMTP 工作階段中批量發送
   - 多處理程序並行發送

2. **附件模式** (`--attachment`)
   - 將單一檔案作為郵件附件發送
   - 自動偵測 MIME 類型
   - 支援主旨/內文範本

3. **附件目錄模式** (`--attachment-dir`)
   - 將目錄中的每個檔案作為獨立郵件發送
   - 與單一附件模式相同的範本支援

**連線處理：**
- 明文連線（連接埠 25）
- STARTTLS（連接埠 587）
- 隱式 TLS（連接埠 465）
- SMTP 驗證（使用者名稱/密碼）
- 連線逾時和重試邏輯
- 連線問題偵測（421 錯誤、管線中斷）

#### stats.rs
- `Stats` 結構體用於追蹤：
  - 郵件數量（總數、成功、失敗）
  - 解析/發送耗時
  - 錯誤分類和檔案清單
  - QPS（每秒查詢數）計算
- 實作 `Display` trait 用於格式化輸出

#### anonymizer.rs
- 將郵箱位址替換為隨機字串
- 保持一致性（相同郵箱 → 相同替換結果）
- 使用 HashMap 快取

### 3. rsendmail-cli

命令列介面應用程式。

**功能：**
- 30+ 命令列選項
- 本地化的 `--help` 輸出
- 優雅關閉（Ctrl+C 處理）
- 迴圈和重複模式
- 可選的日誌檔案輸出
- 失敗郵件儲存

**架構：**
- 使用 clap builder 模式（非 derive）實作執行階段 i18n
- 在 CLI 解析前進行語言偵測
- Tokio 非同步執行環境

### 4. rsendmail-gui

使用 Slint 框架的圖形使用者介面。

**功能：**
- 視覺化 SMTP 設定
- 三種發送模式，帶模式專用 UI
- 即時進度和統計資訊
- 日誌檢視和匯出
- 設定儲存/載入（JSON）
- 語言切換器
- 自訂日誌器，雙輸出（終端機 + GUI）

**UI 元件：**
- 帶分頁的主視窗
- SMTP 伺服器設定面板
- 發送模式選擇器
- 進階選項面板
- 統計資訊顯示
- 日誌檢視器

## 資料流程

### CLI 流程
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
  └─► 迴圈:
        │
        ├─► mailer.send_all_with_cancel(running)
        │     │
        │     ├─► EML 模式: collect_email_files() → send_fixed_mode()
        │     ├─► 附件模式: send_attachment_with_cancel()
        │     └─► 附件目錄模式: send_attachment_dir_with_cancel()
        │
        ├─► 累計 Stats
        │
        └─► 等待下次迭代（如果是迴圈/重複模式）
```

### GUI 流程
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
  │     ├─► on_start_send() ──► 啟動非同步工作
  │     │     │
  │     │     └─► Mailer::send_all_with_cancel()
  │     │           │
  │     │           └─► 透過 mpsc channel 發送事件
  │     │
  │     ├─► on_stop_send() ──► 設定 running = false
  │     │
  │     ├─► on_browse_*() ──► 檔案對話方塊
  │     │
  │     └─► on_save/load_config() ──► JSON 序列化/反序列化
  │
  └─► app.run()
```

## 主要相依性

| 相依性 | 用途 |
|--------|------|
| tokio | 非同步執行環境 |
| mail-send | SMTP 用戶端 |
| mail-parser | EML 檔案解析 |
| mail-builder | 郵件建構 |
| clap | CLI 參數解析 |
| slint | GUI 框架 |
| rust-i18n | 國際化 |
| serde | 設定序列化 |
| walkdir | 目錄遍歷 |
| infer | MIME 類型偵測 |

## 錯誤處理

- `anyhow::Result` 用於應用程式層級錯誤
- `Stats.increment_error()` 用於每封郵件的錯誤追蹤
- 按類型分類錯誤（連線、驗證、發送、解析）
- 儲存失敗郵件檔案以供後續分析

## 執行緒安全

- `Arc<AtomicBool>` 用於優雅關閉訊號
- `Arc<Mutex<...>>` 用於 GUI 日誌器的共用狀態
- Tokio channel 用於 GUI 事件通訊
- 多處理程序模式下每個處理程序獨立的統計資訊

## 設定

`Config` 結構體支援：
- 程式碼中直接存取欄位
- JSON 序列化用於 GUI 儲存/載入
- CLI 參數解析
- 所有可選欄位都有預設值
