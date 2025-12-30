# RSendMail GUI - 简体中文 (zh-CN)
# Fluent Translation File

## Application
app-title = RSendMail

## SMTP Configuration
smtp-server = SMTP 服务器
server-address = 服务器地址
port = 端口
use-tls = 使用 TLS
accept-invalid-certs = 接受自签名证书
auth-required = 需要认证
username = 用户名
password = 密码
sender = 发件人
recipient = 收件人
recipient-hint = (多个地址请用逗号分隔)

## Send Mode
send-mode = 发送模式
eml-batch = EML 批量
single-attachment = 单个附件
dir-attachment = 目录附件
eml-directory = EML 目录
attachment-file = 附件文件
attachment-directory = 附件目录
extension = 扩展名
browse = 浏览...
found-files = 找到 { $count } 个文件
email-subject = 邮件主题
email-body = 邮件正文
filename-hint = 使用 {"{filename}"} 自动插入文件名

## Advanced Options
advanced-options = 高级选项
performance = 性能配置
processes = 进程数
batch-size = 批量大小
send-interval = 发送间隔(ms)
timeout = 超时(秒)
loop-settings = 循环发送
infinite-loop = 无限循环
repeat-count = 重复次数
loop-interval = 循环间隔(秒)
retry-interval = 重试间隔(秒)
email-processing = 邮件处理
keep-headers = 保留原始邮件头
modify-headers = 修改邮件头
anonymize-emails = 匿名化邮箱
domain = 域名
logging = 日志与错误处理
log-level = 日志级别
log-file = 日志文件
failed-emails-dir = 失败邮件
optional = (可选)

## Statistics
statistics = 发送统计
total = 总计
success = 成功
failed = 失败
qps = QPS
current-round = 当前轮次
elapsed-time = 已用时间

## Logs
send-log = 发送日志
clear = 清空
export-log = 导出日志

## Status
status-ready = 就绪
status-preparing = 准备中...
status-sending = 发送中...
status-stopped = 已停止
status-completed = 完成
status-stopping = 停止中...
status-error = 错误

## Buttons
save-config = 保存配置
load-config = 加载配置
test-connection = 测试连接
start-send = 开始发送
stop-send = 停止发送

## Messages - Errors
error-smtp-required = 请输入 SMTP 服务器地址
error-sender-required = 请输入发件人地址
error-recipient-required = 请输入收件人地址
error-eml-dir-required = 请选择 EML 文件目录
error-attachment-required = 请选择附件文件
error-attachment-dir-required = 请选择附件目录
error-username-required = 认证模式需要输入用户名
error-password-required = 认证模式需要输入密码

## Messages - Info
info-testing-connection = 开始测试连接...
info-connecting = 连接到 { $server }:{ $port } (TLS: { $tls })
info-connection-success = 连接测试成功
info-connection-failed = 连接测试失败: { $error }
info-starting-round = 开始第 { $current }/{ $total } 轮发送
info-round-completed = 第 { $round } 轮发送完成
info-send-completed = 发送完成！成功: { $success }, 失败: { $failed }
info-waiting = 等待 { $seconds } 秒后开始下一轮...
info-stopping = 正在停止发送...
info-config-saved = 配置已保存到: { $path }
info-config-loaded = 配置已加载: { $path }
info-log-exported = 日志已导出到: { $path }
info-save-failed = 保存失败: { $error }
info-load-failed = 加载失败: { $error }
info-export-failed = 导出失败: { $error }
info-parse-failed = 解析失败: { $error }

## Language
language = 语言
