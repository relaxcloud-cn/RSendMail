# RSendMail

用於批量發送郵件的高性能測試工具

[English](README.md) | [简体中文](README_zh.md) | 繁體中文 | [日本語](README_ja.md)

![Release](https://img.shields.io/github/v/release/kpassy/RSendMail?color=blue&include_prereleases)
![License](https://img.shields.io/github/license/kpassy/RSendMail)
![Stars](https://img.shields.io/github/stars/kpassy/RSendMail?style=social)
[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/relaxcloud-cn/RSendMail)

## 介面截圖

| English | 简体中文 |
|---------|----------|
| ![English](assets/screenshots/en.png) | ![简体中文](assets/screenshots/zh-ch.png) |

| 繁體中文 | 日本語 |
|----------|--------|
| ![繁體中文](assets/screenshots/zh-hk.png) | ![日本語](assets/screenshots/ja.png) |

## 功能特點

- **CLI 和 GUI 兩種模式**：支援命令列和圖形介面
- 批量處理和發送多封郵件
- 多執行緒處理提升效能
- 支援自訂 SMTP 伺服器設定
- 支援多級日誌輸出（error/warn/info/debug/trace）
- 詳細的錯誤追蹤和統計資訊
- Docker 支援便於部署
- 支援在單一 SMTP 會話中批量發送
- 支援發送一般檔案作為附件
- 支援批量發送目錄中的所有檔案作為獨立郵件
- **多語言支援**：英文、簡體中文、繁體中文、日文

## 附件功能說明

RSendMail 現在支援將一般檔案作為附件發送，無需先建立 EML 檔案。這對於快速發送檔案測試非常有用。

### 附件模式特點

- 自動偵測檔案 MIME 類型
- 支援自訂郵件主題和內容
- 使用範本變數自動填入檔案名稱
- 可選 HTML 內容支援
- 與批量 EML 發送功能完全獨立
- 支援發送單一檔案（使用 `--attachment`）或目錄中的全部檔案（使用 `--attachment-dir`）
- 在附件模式下不需要提供 `--dir` 參數

### 範本變數

- `{filename}`: 將被替換為實際的檔案名稱（不含路徑）

## 多語言支援

RSendMail 的 CLI 和 GUI 介面均支援多語言：

| 語言 | 代碼 | 環境變數 |
|------|------|----------|
| 英文 | `en` | `RSENDMAIL_LANG=en` |
| 簡體中文 | `zh-CN` | `RSENDMAIL_LANG=zh-CN` |
| 繁體中文 | `zh-TW` | `RSENDMAIL_LANG=zh-TW` |
| 日文 | `ja` | `RSENDMAIL_LANG=ja` |

### 設定語言

**方法一：環境變數**
```bash
# Linux/macOS
export RSENDMAIL_LANG=zh-TW
rsendmail --help

# Windows PowerShell
$env:RSENDMAIL_LANG="zh-TW"
rsendmail --help
```

**方法二：命令列參數**
```bash
rsendmail --lang zh-TW --help
```

**方法三：自動偵測**
如果未指定語言，RSendMail 會自動按以下順序偵測系統語言：
1. `RSENDMAIL_LANG` 環境變數
2. `LANG` 或 `LC_ALL` 環境變數
3. 系統區域設定（macOS: AppleLocale）
4. 如果偵測失敗，預設使用英文

### 語言範例

```bash
# 顯示英文說明
RSENDMAIL_LANG=en rsendmail --help

# 顯示繁體中文說明
RSENDMAIL_LANG=zh-TW rsendmail --help

# 顯示日文說明
rsendmail --lang ja --help
```

## 建置

### 本機建置
```bash
cd rsendmail
cargo build --release
```

### Docker 建置
```bash
docker build -t rsendmail .
```

## 使用方法

### Windows 使用
從 [Releases](https://github.com/kpassy/RSendMail/releases) 頁面下載 Windows 執行檔（`rsendmail-windows-x86_64.exe`）。
```bash
rsendmail-windows-x86_64.exe --smtp-server <smtp伺服器> --port <連接埠> --from <寄件人> --to <收件人> --dir <郵件目錄> --processes <處理程序數> --batch-size <批次大小>
```

### 本機使用
```bash
rsendmail --smtp-server <smtp伺服器> --port <連接埠> --from <寄件人> --to <收件人> --dir <郵件目錄> --processes <處理程序數> --batch-size <批次大小>
```

### Docker 使用
```bash
docker run --rm -v /path/to/emails:/data rsendmail --smtp-server <smtp伺服器> --port <連接埠> --from <寄件人> --to <收件人> --dir /data --processes <處理程序數> --batch-size <批次大小>
```

### 參數說明

- `--smtp-server`: SMTP 伺服器位址
- `--port`: SMTP 伺服器連接埠（預設：25）
- `--from`: 寄件人郵箱位址（用於 SMTP 信封，預設不修改郵件內容）
- `--to`: 收件人郵箱位址（用於 SMTP 信封，多個位址請用逗號分隔，預設不修改郵件內容）
- `--dir`: 郵件檔案所在目錄（僅在使用 EML 發送模式時需要，使用 --attachment 或 --attachment-dir 時不需要）
- `--extension`: 郵件檔案副檔名（預設：eml）
- `--processes`: 處理程序數，auto 表示自動設定為 CPU 核心數，或者指定具體數字（預設：auto）
- `--batch-size`: 每個 SMTP 會話連續發送的郵件數量（預設：1）
- `--smtp-timeout`: SMTP 會話逾時時間（秒）（預設：30）
- `--log-level`: 日誌級別（error/warn/info/debug/trace）（預設：info）
- `--keep-headers`: 保留原始郵件標頭（預設：false，優先級高於 modify-headers）
- `--modify-headers`: 使用 --from 和 --to 參數修改郵件標頭中的 From 和 To（預設：false）
- `--anonymize-emails`: 匿名化所有郵箱位址（預設：false）
- `--anonymize-domain`: 匿名化使用的網域（預設：example.com）
- `--loop`: 無限迴圈發送郵件，直到使用者中斷（預設：false）
- `--repeat`: 重複發送次數（預設：1）
- `--loop-interval`: 迴圈發送的間隔時間（秒）（預設：1）
- `--retry-interval`: 發送失敗後重試的間隔時間（秒）（預設：5）
- `--attachment`: 附件檔案路徑，用於發送一般檔案作為附件
- `--attachment-dir`: 附件目錄路徑，發送目錄下所有檔案為獨立的郵件（每個檔案一封郵件）
- `--subject-template`: 主題範本，支援變數 {filename}（預設："附件: {filename}"）
- `--text-template`: 文字內容範本，支援變數 {filename}（預設："請查收附件: {filename}"）
- `--html-template`: HTML 內容範本，支援變數 {filename}
- `--email-send-interval`: 批量發送時，每封郵件之間的發送間隔（毫秒，預設為 0）
- `--auth-mode`: 啟用郵箱帳號登入模式（透過使用者名稱和密碼驗證發送郵件）
- `--username`: 郵箱帳號使用者名稱（在 auth-mode 啟用時必需）
- `--password`: 郵箱帳號密碼（在 auth-mode 啟用時必需）
- `--use-tls`: 使用 TLS 加密連線（當連接埠為 465 時自動啟用）
- `--accept-invalid-certs`: 接受無效的 TLS 憑證（僅當使用 TLS 時）。警告：這會降低安全性，請僅在信任目標伺服器時使用。
- `--failed-emails-dir`: 發送失敗的 EML 檔案儲存目錄（指定後會自動將失敗的郵件複製到該目錄，檔案名稱會加上時間戳記避免覆蓋）
- `--log-file`: 日誌檔案儲存路徑（如果指定，日誌會同時輸出到主控台和檔案，方便保存執行記錄）
- `--lang`: 顯示語言（en/zh-CN/zh-TW/ja），也可透過環境變數 `RSENDMAIL_LANG` 設定

## 日誌級別

應用程式支援不同的日誌級別來控制輸出的詳細程度：

- `error`: 僅顯示錯誤訊息
- `warn`: 顯示警告和錯誤訊息
- `info`: 顯示一般進度資訊（預設）
- `debug`: 顯示詳細的除錯資訊
- `trace`: 顯示最詳細的追蹤資訊

## 使用範例

```bash
# 預設日誌級別（info）
rsendmail --smtp-server 192.168.1.100 --port 25 --from sender@example.com --to recipient@example.com --dir ./emails --processes 10 --batch-size 5

# 詳細除錯日誌
rsendmail --smtp-server 192.168.1.100 --port 25 --from sender@example.com --to recipient@example.com --dir ./emails --processes 10 --batch-size 5 --log-level debug

# 僅顯示錯誤訊息
rsendmail --smtp-server 192.168.1.100 --port 25 --from sender@example.com --to recipient@example.com --dir ./emails --processes 10 --batch-size 5 --log-level error

# Docker 執行範例
docker run --rm -v $(pwd)/emails:/data rsendmail --smtp-server 192.168.1.100 --port 25 --from sender@example.com --to recipient@example.com --dir /data --processes 10 --batch-size 5 --log-level info

# 發送單一附件範例（不需要 --dir 參數）
rsendmail --smtp-server 192.168.1.100 --port 25 --from sender@example.com --to recipient@example.com --attachment ./document.pdf

# 使用自訂範本發送附件（不需要 --dir 參數）
rsendmail --smtp-server 192.168.1.100 --port 25 --from sender@example.com --to recipient@example.com --attachment ./document.pdf --subject-template "重要檔案: {filename}" --text-template "您好，\n\n請查收附件：{filename}。\n\n此致，\nRSendMail團隊"

# 批量發送目錄中的所有檔案為獨立的郵件（不需要 --dir 參數）
rsendmail --smtp-server 192.168.1.100 --port 25 --from sender@example.com --to recipient@example.com --attachment-dir ./documents --subject-template "檔案: {filename}"
```

## 文件

- [架構設計](docs/ARCHITECTURE_zh-TW.md) - 詳細的架構和設計文件

## 授權條款

MIT License
