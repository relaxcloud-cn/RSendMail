# RSendMail GUI - 日本語 (ja-JP)
# Fluent Translation File

## Application
app-title = RSendMail

## SMTP Configuration
smtp-server = SMTP サーバー
server-address = サーバーアドレス
port = ポート
use-tls = TLS を使用
accept-invalid-certs = 自己署名証明書を許可
auth-required = 認証が必要
username = ユーザー名
password = パスワード
sender = 送信者
recipient = 受信者
recipient-hint = (複数はカンマ区切り)

## Send Mode
send-mode = 送信モード
eml-batch = EML 一括
single-attachment = 単一添付
dir-attachment = フォルダ添付
eml-directory = EML フォルダ
attachment-file = 添付ファイル
attachment-directory = 添付フォルダ
extension = 拡張子
browse = 参照...
found-files = { $count } 件のファイルを検出
email-subject = メール件名
email-body = メール本文
filename-hint = {"{filename}"} でファイル名を自動挿入

## Advanced Options
advanced-options = 詳細オプション
performance = パフォーマンス設定
processes = プロセス数
batch-size = バッチサイズ
send-interval = 送信間隔(ms)
timeout = タイムアウト(秒)
loop-settings = ループ送信
infinite-loop = 無限ループ
repeat-count = 繰り返し回数
loop-interval = ループ間隔(秒)
retry-interval = リトライ間隔(秒)
email-processing = メール処理
keep-headers = 元のヘッダーを保持
modify-headers = ヘッダーを変更
anonymize-emails = メールを匿名化
domain = ドメイン
logging = ログとエラー処理
log-level = ログレベル
log-file = ログファイル
failed-emails-dir = 失敗メール
optional = (任意)

## Statistics
statistics = 送信統計
total = 合計
success = 成功
failed = 失敗
qps = QPS
current-round = 現在のラウンド
elapsed-time = 経過時間

## Logs
send-log = 送信ログ
clear = クリア
export-log = ログをエクスポート

## Status
status-ready = 準備完了
status-preparing = 準備中...
status-sending = 送信中...
status-stopped = 停止
status-completed = 完了
status-stopping = 停止中...
status-error = エラー

## Buttons
save-config = 設定を保存
load-config = 設定を読み込み
test-connection = 接続テスト
start-send = 送信開始
stop-send = 送信停止

## Messages - Errors
error-smtp-required = SMTP サーバーアドレスを入力してください
error-sender-required = 送信者アドレスを入力してください
error-recipient-required = 受信者アドレスを入力してください
error-eml-dir-required = EML フォルダを選択してください
error-attachment-required = 添付ファイルを選択してください
error-attachment-dir-required = 添付フォルダを選択してください
error-username-required = 認証にはユーザー名が必要です
error-password-required = 認証にはパスワードが必要です

## Messages - Info
info-testing-connection = 接続をテスト中...
info-connecting = { $server }:{ $port } に接続中 (TLS: { $tls })
info-connection-success = 接続テスト成功
info-connection-failed = 接続テスト失敗: { $error }
info-starting-round = ラウンド { $current }/{ $total } を開始
info-round-completed = ラウンド { $round } 完了
info-send-completed = 送信完了！成功: { $success }, 失敗: { $failed }
info-waiting = { $seconds } 秒後に次のラウンドを開始...
info-stopping = 送信を停止中...
info-config-saved = 設定を保存しました: { $path }
info-config-loaded = 設定を読み込みました: { $path }
info-log-exported = ログをエクスポートしました: { $path }
info-save-failed = 保存に失敗: { $error }
info-load-failed = 読み込みに失敗: { $error }
info-export-failed = エクスポートに失敗: { $error }
info-parse-failed = 解析に失敗: { $error }

## Language
language = 言語
