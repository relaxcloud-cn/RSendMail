# RSendMail

用于批量发送邮件的高性能测试工具

[English](README.md) | 简体中文

![Release](https://img.shields.io/github/v/release/kpassy/RSendMail?color=blue&include_prereleases)
![License](https://img.shields.io/github/license/kpassy/RSendMail)
![Stars](https://img.shields.io/github/stars/kpassy/RSendMail?style=social)
[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/relaxcloud-cn/RSendMail)

## 功能特点

- 批量处理和发送多个邮件
- 多线程处理提升性能
- 支持自定义 SMTP 服务器配置
- 支持多级日志输出（error/warn/info/debug/trace）
- 详细的错误跟踪和统计信息
- Docker 支持便于部署
- 支持在单个 SMTP 会话中批量发送
- 支持发送普通文件作为附件
- 支持批量发送目录中的所有文件作为单独的邮件

## 附件功能说明

RSendMail现在支持将普通文件作为附件发送，无需先创建EML文件。这对于快速发送文件测试非常有用。

### 附件模式特点

- 自动检测文件MIME类型
- 支持自定义邮件主题和内容
- 使用模板变量自动填充文件名
- 可选HTML内容支持
- 与批量EML发送功能完全独立
- 支持发送单个文件（使用`--attachment`）或目录中的全部文件（使用`--attachment-dir`）
- 在附件模式下不需要提供`--dir`参数

### 模板变量

## 构建

### 本地构建
```bash
cd rsendmail
cargo build --release
```

### Docker 构建
```bash
docker build -t rsendmail .
```

## 使用方法

### Windows 使用
从 [Releases](https://github.com/kpassy/RSendMail/releases) 页面下载Windows可执行文件（`rsendmail-windows-x86_64.exe`）。
```bash
rsendmail-windows-x86_64.exe --smtp-server <smtp服务器> --port <端口> --from <发件人> --to <收件人> --dir <邮件目录> --processes <进程数> --batch-size <批处理大小>
```

### 本地使用
```bash
rsendmail --smtp-server <smtp服务器> --port <端口> --from <发件人> --to <收件人> --dir <邮件目录> --processes <进程数> --batch-size <批处理大小>
```

### Docker 使用
```bash
docker run --rm -v /path/to/emails:/data rsendmail --smtp-server <smtp服务器> --port <端口> --from <发件人> --to <收件人> --dir /data --processes <进程数> --batch-size <批处理大小>
```

### 参数说明

- `--smtp-server`: SMTP 服务器地址
- `--port`: SMTP 服务器端口（默认：25）
- `--from`: 发件人邮箱地址（用于SMTP信封，默认不修改邮件内容）
- `--to`: 收件人邮箱地址（用于SMTP信封，多个地址请用逗号分隔，默认不修改邮件内容）
- `--dir`: 邮件文件所在目录（仅在使用EML发送模式时需要，使用--attachment或--attachment-dir时不需要）
- `--extension`: 邮件文件扩展名（默认：eml）
- `--processes`: 进程数，auto表示自动设置为CPU核心数，或者指定具体数字（默认：auto）
- `--batch-size`: 每个SMTP会话连续发送的邮件数量（默认：1）
- `--smtp-timeout`: SMTP会话超时时间（秒）（默认：30）
- `--log-level`: 日志级别（error/warn/info/debug/trace）（默认：info）
- `--keep-headers`: 保留原始邮件头（默认：false，优先级高于modify-headers）
- `--modify-headers`: 使用--from和--to参数修改邮件头中的From和To（默认：false）
- `--anonymize-emails`: 匿名化所有邮箱地址（默认：false）
- `--anonymize-domain`: 匿名化使用的域名（默认：example.com）
- `--loop`: 无限循环发送邮件，直到用户中断（默认：false）
- `--repeat`: 重复发送次数（默认：1）
- `--loop-interval`: 循环发送的间隔时间（秒）（默认：1）
- `--retry-interval`: 发送失败后重试的间隔时间（秒）（默认：5）
- `--attachment`: 附件文件路径，用于发送普通文件作为附件
- `--attachment-dir`: 附件目录路径，发送目录下所有文件为单独的邮件（每个文件一封邮件）
- `--subject-template`: 主题模板，支持变量{filename}（默认："附件: {filename}"）
- `--text-template`: 文本内容模板，支持变量{filename}（默认："请查收附件: {filename}"）
- `--html-template`: HTML内容模板，支持变量{filename}
- `--email-send-interval`: 批量发送时，每封邮件之间的发送间隔（毫秒，默认为0）
- `--auth-mode`: 启用邮箱账号登录模式（通过用户名和密码验证发送邮件）
- `--username`: 邮箱账号用户名（在auth-mode启用时必需）
- `--password`: 邮箱账号密码（在auth-mode启用时必需）
- `--use-tls`: 使用TLS加密连接（当端口为465时自动启用）
- `--email-send-interval-ms`: 每个邮件处理批次（当批次大小为1时，即每个文件处理尝试）完成后的等待毫秒数。这在连接失败、认证失败或邮件发送失败/成功后都会生效。默认值为0（无等待）。
- `--smtp-timeout`: SMTP操作的超时时间（秒）。默认值为60。
- `--use-tls`: 如果设置，将尝试使用TLS (STARTTLS) 连接SMTP服务器（25或587端口）。如果端口是465，则总是使用隐式TLS。
- `--accept-invalid-certs`: 接受无效的TLS证书（仅当使用TLS时）。警告：这会降低安全性，请仅在信任目标服务器时使用。

## 日志级别

应用程序支持不同的日志级别来控制输出的详细程度：

- `error`: 仅显示错误信息
- `warn`: 显示警告和错误信息
- `info`: 显示一般进度信息（默认）
- `debug`: 显示详细的调试信息
- `trace`: 显示最详细的跟踪信息

## 使用示例

```bash
# 默认日志级别（info）
rsendmail --smtp-server 192.168.1.100 --port 25 --from sender@example.com --to recipient@example.com --dir ./emails --processes 10 --batch-size 5

# 详细调试日志
rsendmail --smtp-server 192.168.1.100 --port 25 --from sender@example.com --to recipient@example.com --dir ./emails --processes 10 --batch-size 5 --log-level debug

# 仅显示错误信息
rsendmail --smtp-server 192.168.1.100 --port 25 --from sender@example.com --to recipient@example.com --dir ./emails --processes 10 --batch-size 5 --log-level error

# Docker 运行示例
docker run --rm -v $(pwd)/emails:/data rsendmail --smtp-server 192.168.1.100 --port 25 --from sender@example.com --to recipient@example.com --dir /data --processes 10 --batch-size 5 --log-level info

# 发送单个附件示例（不需要--dir参数）
rsendmail --smtp-server 192.168.1.100 --port 25 --from sender@example.com --to recipient@example.com --attachment ./document.pdf

# 使用自定义模板发送附件（不需要--dir参数）
rsendmail --smtp-server 192.168.1.100 --port 25 --from sender@example.com --to recipient@example.com --attachment ./document.pdf --subject-template "重要文件: {filename}" --text-template "您好，\n\n请查收附件：{filename}。\n\n此致，\nRSendMail团队"

# 批量发送目录中的所有文件为独立的邮件（不需要--dir参数）
rsendmail --smtp-server 192.168.1.100 --port 25 --from sender@example.com --to recipient@example.com --attachment-dir ./documents --subject-template "文件: {filename}"
```