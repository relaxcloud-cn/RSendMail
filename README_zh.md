# RSendMail

用于批量发送邮件的高性能测试工具

[English](README.md) | 简体中文

## 功能特点

- 批量处理和发送多个邮件
- 多线程处理提升性能
- 支持自定义 SMTP 服务器配置
- 支持多级日志输出（error/warn/info/debug/trace）
- 详细的错误跟踪和统计信息
- Docker 支持便于部署
- 支持在单个 SMTP 会话中批量发送

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
```
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
- `--from`: 发件人邮箱地址
- `--to`: 收件人邮箱地址
- `--dir`: 邮件文件所在目录
- `--extension`: 邮件文件扩展名（默认：eml）
- `--processes`: 进程数，auto表示自动设置为CPU核心数，或者指定具体数字（默认：auto）
- `--batch-size`: 每个SMTP会话连续发送的邮件数量（默认：1）
- `--smtp-timeout`: SMTP会话超时时间（秒）（默认：30）
- `--log-level`: 日志级别（error/warn/info/debug/trace）（默认：info）
- `--keep-headers`: 保留原始邮件头（默认：false）

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
```

## Docker 容器说明

Docker 容器设计考虑了安全性和效率：

- 基于 debian:bookworm-slim，保持最小体积
- 以非 root 用户运行
- 仅包含必要的运行时依赖
- 使用卷挂载邮件文件
- 无状态操作

### 容器结构

- `/usr/local/bin/rsendmail`: 应用程序二进制文件
- `/data`: 邮件文件挂载点（卷）

## 性能特点

- 多线程并行处理
- 高效的内存使用
- 快速的邮件解析和发送
- 详细的性能统计输出
- 支持批量发送以提高效率

## 安全特性

- 非 root 用户执行
- 最小容器体积
- 隔离的运行环境
- 无持久存储
