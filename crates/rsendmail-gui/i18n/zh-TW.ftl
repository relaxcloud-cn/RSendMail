# RSendMail GUI - 繁體中文 (zh-TW)
# Fluent Translation File

## Application
app-title = RSendMail

## SMTP Configuration
smtp-server = SMTP 伺服器
server-address = 伺服器地址
port = 連接埠
use-tls = 使用 TLS
accept-invalid-certs = 接受自簽名憑證
auth-required = 需要認證
username = 使用者名稱
password = 密碼
sender = 寄件人
recipient = 收件人
recipient-hint = (多個地址請用逗號分隔)

## Send Mode
send-mode = 發送模式
eml-batch = EML 批次
single-attachment = 單一附件
dir-attachment = 目錄附件
eml-directory = EML 目錄
attachment-file = 附件檔案
attachment-directory = 附件目錄
extension = 副檔名
browse = 瀏覽...
found-files = 找到 { $count } 個檔案
email-subject = 郵件主旨
email-body = 郵件內文
filename-hint = 使用 {"{filename}"} 自動插入檔案名稱

## Advanced Options
advanced-options = 進階選項
performance = 效能設定
processes = 處理程序數
batch-size = 批次大小
send-interval = 發送間隔(ms)
timeout = 逾時(秒)
loop-settings = 循環發送
infinite-loop = 無限循環
repeat-count = 重複次數
loop-interval = 循環間隔(秒)
retry-interval = 重試間隔(秒)
email-processing = 郵件處理
keep-headers = 保留原始郵件標頭
modify-headers = 修改郵件標頭
anonymize-emails = 匿名化郵箱
domain = 網域
logging = 日誌與錯誤處理
log-level = 日誌等級
log-file = 日誌檔案
failed-emails-dir = 失敗郵件
optional = (選填)

## Statistics
statistics = 發送統計
total = 總計
success = 成功
failed = 失敗
qps = QPS
current-round = 目前輪次
elapsed-time = 已用時間

## Logs
send-log = 發送日誌
clear = 清空
export-log = 匯出日誌

## Status
status-ready = 就緒
status-preparing = 準備中...
status-sending = 發送中...
status-stopped = 已停止
status-completed = 完成
status-stopping = 停止中...
status-error = 錯誤

## Buttons
save-config = 儲存設定
load-config = 載入設定
test-connection = 測試連線
start-send = 開始發送
stop-send = 停止發送

## Messages - Errors
error-smtp-required = 請輸入 SMTP 伺服器地址
error-sender-required = 請輸入寄件人地址
error-recipient-required = 請輸入收件人地址
error-eml-dir-required = 請選擇 EML 檔案目錄
error-attachment-required = 請選擇附件檔案
error-attachment-dir-required = 請選擇附件目錄
error-username-required = 認證模式需要輸入使用者名稱
error-password-required = 認證模式需要輸入密碼

## Messages - Info
info-testing-connection = 開始測試連線...
info-connecting = 連接到 { $server }:{ $port } (TLS: { $tls })
info-connection-success = 連線測試成功
info-connection-failed = 連線測試失敗: { $error }
info-starting-round = 開始第 { $current }/{ $total } 輪發送
info-round-completed = 第 { $round } 輪發送完成
info-send-completed = 發送完成！成功: { $success }, 失敗: { $failed }
info-waiting = 等待 { $seconds } 秒後開始下一輪...
info-stopping = 正在停止發送...
info-config-saved = 設定已儲存至: { $path }
info-config-loaded = 設定已載入: { $path }
info-log-exported = 日誌已匯出至: { $path }
info-save-failed = 儲存失敗: { $error }
info-load-failed = 載入失敗: { $error }
info-export-failed = 匯出失敗: { $error }
info-parse-failed = 解析失敗: { $error }

## Language
language = 語言
