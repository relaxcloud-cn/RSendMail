# RSendMail 架构设计

[English](ARCHITECTURE.md) | 简体中文 | [繁體中文](ARCHITECTURE_zh-TW.md) | [日本語](ARCHITECTURE_ja.md)

本文档描述 RSendMail 的架构设计，这是一个高性能的批量邮件发送工具。

## 概述

RSendMail 是一个基于 Rust 的应用程序，用于通过 SMTP 测试和发送批量邮件。它提供 CLI 和 GUI 两种界面，共享一个核心库。

```
┌─────────────────────────────────────────────────────────┐
│                      应用层                              │
├──────────────────────┬──────────────────────────────────┤
│    rsendmail-cli     │         rsendmail-gui            │
│     (命令行工具)      │        (Slint 图形界面)           │
├──────────────────────┴──────────────────────────────────┤
│                   rsendmail-core                         │
│                  (邮件发送引擎)                           │
├─────────────────────────────────────────────────────────┤
│                   rsendmail-i18n                         │
│                   (国际化支持)                            │
└─────────────────────────────────────────────────────────┘
```

## 项目结构

```
RSendMail/
├── Cargo.toml                      # 工作空间配置
├── crates/
│   ├── rsendmail-i18n/             # 国际化支持
│   │   ├── src/lib.rs              # Language 枚举，tr() 函数
│   │   └── locales/                # YAML 翻译文件
│   │       ├── en-US.yml           # 英文 (默认)
│   │       ├── zh-CN.yml           # 简体中文
│   │       ├── zh-TW.yml           # 繁体中文
│   │       └── ja-JP.yml           # 日文
│   │
│   ├── rsendmail-core/             # 核心库
│   │   └── src/
│   │       ├── lib.rs              # 库导出
│   │       ├── config.rs           # 配置结构体
│   │       ├── mailer.rs           # 邮件发送引擎 (~1800 行)
│   │       ├── stats.rs            # 统计信息收集
│   │       └── anonymizer.rs       # 邮箱匿名化
│   │
│   ├── rsendmail-cli/              # CLI 应用
│   │   └── src/
│   │       ├── main.rs             # 入口，循环控制
│   │       ├── args.rs             # CLI 参数解析 (clap builder)
│   │       └── logging.rs          # 日志初始化
│   │
│   └── rsendmail-gui/              # GUI 应用
│       ├── src/
│       │   ├── main.rs             # GUI 入口
│       │   └── i18n.rs             # GUI 专用 i18n
│       ├── ui/
│       │   └── app.slint           # UI 定义
│       └── fonts/                  # 自定义字体
│
├── assets/
│   └── screenshots/                # GUI 截图
│
└── docs/
    └── ARCHITECTURE.md             # 本文档
```

## Crate 依赖关系

```
rsendmail-cli ──┬──► rsendmail-core ──► rsendmail-i18n
                │
                └──► rsendmail-i18n

rsendmail-gui ──┬──► rsendmail-core ──► rsendmail-i18n
                │
                └──► (GUI 使用独立的 HashMap 方式实现 i18n)
```

## 核心组件

### 1. rsendmail-i18n

使用 `rust-i18n` 库的共享国际化模块。

**功能：**
- 支持 4 种语言：英文、简体中文、繁体中文、日文
- 从环境变量和系统区域设置检测语言
- 翻译函数：`tr(key)` 和 `tr_with_args(key, args)`
- 基于 YAML 的翻译文件（每种语言约 250 个键）

**语言检测优先级：**
1. `--lang` CLI 参数
2. `RSENDMAIL_LANG` 环境变量
3. `LANG` / `LC_ALL` 环境变量
4. macOS `AppleLocale`（仅 macOS）
5. 默认使用英文

### 2. rsendmail-core

核心邮件发送引擎，CLI 和 GUI 共享使用。

**模块：**

#### config.rs
- `Config` 结构体，包含 30+ 配置选项
- 支持 Serde 序列化（用于保存/加载）
- 所有可选字段都有默认值
- `ProcessMode` 枚举（Auto / Fixed）

#### mailer.rs (~1800 行)
主要的邮件发送逻辑，支持三种操作模式：

1. **EML 模式** (`--dir`)
   - 从目录读取 EML 文件
   - 支持单个 SMTP 会话中批量发送
   - 多进程并行发送

2. **附件模式** (`--attachment`)
   - 将单个文件作为邮件附件发送
   - 自动检测 MIME 类型
   - 支持主题/正文模板

3. **附件目录模式** (`--attachment-dir`)
   - 将目录中的每个文件作为单独邮件发送
   - 与单附件模式相同的模板支持

**连接处理：**
- 明文连接（端口 25）
- STARTTLS（端口 587）
- 隐式 TLS（端口 465）
- SMTP 认证（用户名/密码）
- 连接超时和重试逻辑
- 连接问题检测（421 错误、管道断开）

#### stats.rs
- `Stats` 结构体用于跟踪：
  - 邮件数量（总数、成功、失败）
  - 解析/发送耗时
  - 错误分类和文件列表
  - QPS（每秒查询数）计算
- 实现 `Display` trait 用于格式化输出

#### anonymizer.rs
- 将邮箱地址替换为随机字符串
- 保持一致性（相同邮箱 → 相同替换结果）
- 使用 HashMap 缓存

### 3. rsendmail-cli

命令行界面应用程序。

**功能：**
- 30+ 命令行选项
- 本地化的 `--help` 输出
- 优雅关闭（Ctrl+C 处理）
- 循环和重复模式
- 可选的日志文件输出
- 失败邮件保存

**架构：**
- 使用 clap builder 模式（非 derive）实现运行时 i18n
- 在 CLI 解析前进行语言检测
- Tokio 异步运行时

### 4. rsendmail-gui

使用 Slint 框架的图形用户界面。

**功能：**
- 可视化 SMTP 配置
- 三种发送模式，带模式专用 UI
- 实时进度和统计信息
- 日志查看和导出
- 配置保存/加载（JSON）
- 语言切换器
- 自定义日志器，双输出（终端 + GUI）

**UI 组件：**
- 带标签页的主窗口
- SMTP 服务器设置面板
- 发送模式选择器
- 高级选项面板
- 统计信息显示
- 日志查看器

## 数据流

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
  └─► 循环:
        │
        ├─► mailer.send_all_with_cancel(running)
        │     │
        │     ├─► EML 模式: collect_email_files() → send_fixed_mode()
        │     ├─► 附件模式: send_attachment_with_cancel()
        │     └─► 附件目录模式: send_attachment_dir_with_cancel()
        │
        ├─► 累计 Stats
        │
        └─► 等待下次迭代（如果是循环/重复模式）
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
  │     ├─► on_start_send() ──► 启动异步任务
  │     │     │
  │     │     └─► Mailer::send_all_with_cancel()
  │     │           │
  │     │           └─► 通过 mpsc channel 发送事件
  │     │
  │     ├─► on_stop_send() ──► 设置 running = false
  │     │
  │     ├─► on_browse_*() ──► 文件对话框
  │     │
  │     └─► on_save/load_config() ──► JSON 序列化/反序列化
  │
  └─► app.run()
```

## 主要依赖

| 依赖 | 用途 |
|------|------|
| tokio | 异步运行时 |
| mail-send | SMTP 客户端 |
| mail-parser | EML 文件解析 |
| mail-builder | 邮件构建 |
| clap | CLI 参数解析 |
| slint | GUI 框架 |
| rust-i18n | 国际化 |
| serde | 配置序列化 |
| walkdir | 目录遍历 |
| infer | MIME 类型检测 |

## 错误处理

- `anyhow::Result` 用于应用级错误
- `Stats.increment_error()` 用于每封邮件的错误跟踪
- 按类型分类错误（连接、认证、发送、解析）
- 保存失败邮件文件以供后续分析

## 线程安全

- `Arc<AtomicBool>` 用于优雅关闭信号
- `Arc<Mutex<...>>` 用于 GUI 日志器的共享状态
- Tokio channel 用于 GUI 事件通信
- 多进程模式下每个进程独立的统计信息

## 配置

`Config` 结构体支持：
- 代码中直接访问字段
- JSON 序列化用于 GUI 保存/加载
- CLI 参数解析
- 所有可选字段都有默认值
