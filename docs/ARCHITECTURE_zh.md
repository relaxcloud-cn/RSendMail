# RSendMail 架构说明

[English](ARCHITECTURE.md) | 简体中文 | [繁體中文](ARCHITECTURE_zh-TW.md) | [日本語](ARCHITECTURE_ja.md)

本文档描述当前 RSendMail 在统一到 Tauri GUI 之后的整体架构。

## 概览

RSendMail 现在保留两个用户入口：

- `rsendmail-cli`：命令行工具
- `rsendmail-tauri`：桌面图形界面

两者共用同一个 `rsendmail-core` 业务核心，因此邮件解析、SMTP 发送、重试、附件处理和统计逻辑保持一致。

```text
┌────────────────────────────────────────────────────────────┐
│                         应用入口                           │
├───────────────────────────┬────────────────────────────────┤
│      rsendmail-cli        │        rsendmail-tauri        │
│        Rust CLI           │   Tauri + Vue 桌面 GUI        │
├───────────────────────────┴────────────────────────────────┤
│                      rsendmail-core                        │
│         共享的发送器、配置、统计与匿名化逻辑               │
├────────────────────────────────────────────────────────────┤
│                      rsendmail-i18n                        │
│                  Rust 侧共享国际化资源                     │
└────────────────────────────────────────────────────────────┘
```

## 项目结构

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
│       ├── src-tauri/              # Rust Tauri 外壳与命令
│       ├── package.json
│       └── vite.config.ts
├── assets/
│   └── screenshots/
└── docs/
    └── ARCHITECTURE_zh.md
```

## 依赖关系

```text
rsendmail-cli ─────► rsendmail-core ─────► rsendmail-i18n

rsendmail-tauri
  ├── 前端层（Vue、vue-i18n、@tauri-apps/api）
  └── src-tauri Rust 外壳 ───────────────► rsendmail-core
```

## 核心组件

### 1. `rsendmail-i18n`

基于 `rust-i18n` 的 Rust 侧共享国际化模块。

- 翻译文件位于 `crates/rsendmail-i18n/locales/`
- 负责 Rust 代码的语言检测
- 提供 `tr()` 和 `tr_with_args()` 接口

### 2. `rsendmail-core`

CLI 和 GUI 共用的邮件发送核心。

- `config.rs`：可序列化的运行配置
- `mailer.rs`：EML 发送、附件发送、重试、SMTP 会话处理
- `stats.rs`：计数、耗时、吞吐和报告
- `anonymizer.rs`：邮箱地址匿名化

### 3. `rsendmail-cli`

面向自动化和脚本场景的 Rust 命令行程序。

- 使用 `clap` 解析本地化参数
- 初始化日志和可选日志文件输出
- 在 `Mailer` 之上执行循环和重复发送
- 保持现有 CLI 行为稳定

### 4. `rsendmail-tauri`

桌面 GUI 分为两层：

- `src/`：Vue 3 + TypeScript + Vite 界面
- `src-tauri/`：Rust Tauri 外壳，向前端暴露命令

前端职责：

- 图形化收集 SMTP 和发送模式配置
- 通过 Tauri 命令启动和停止发送
- 监听 Rust 发出的日志、进度和统计事件
- 通过 `vue-i18n` 渲染多语言界面

Rust 外壳职责：

- 接收前端传入的 `Config`
- 直接复用 `rsendmail-core::Mailer`
- 通过 Tauri 事件把运行日志和状态推送给 GUI
- 用 `Arc<AtomicBool>` 维护运行状态

## 数据流

### CLI 流程

```text
CLI 参数
  -> rsendmail-cli
  -> Config
  -> rsendmail-core::Mailer
  -> SMTP / 文件系统操作
  -> 日志与统计输出
```

### GUI 流程

```text
Vue 界面
  -> invoke("start_sending", Config)
  -> Tauri 命令处理器
  -> rsendmail-core::Mailer
  -> 发出日志 / 进度 / 统计事件
  -> Vue 监听器更新桌面界面
```

## 关键依赖

| 依赖 | 用途 |
|------|------|
| `tokio` | 异步运行时 |
| `mail-send` | SMTP 客户端 |
| `mail-parser` | EML 解析 |
| `mail-builder` | 邮件与附件构造 |
| `clap` | CLI 参数解析 |
| `tauri` | 桌面应用外壳 |
| `vue` | GUI 组件运行时 |
| `vite` | 前端构建工具 |
| `vue-i18n` | GUI 国际化 |
| `serde` / `serde_json` | 共享配置序列化 |

## 并发与状态

- `Arc<AtomicBool>` 控制发送任务的启动和停止
- `Mutex<Option<AppHandle>>` 让 Rust 日志器把消息转发到 GUI
- Tokio 任务保证发送过程中桌面界面仍保持响应

## 配置模型

`Config` 是 CLI 与 GUI 共享的契约。

- CLI 通过参数解析生成它
- Tauri GUI 从 Vue 前端传入 Rust 命令
- Serde 保证保存、加载和互操作稳定

## 维护约束

- 除非明确变更需求，否则不要破坏 CLI 行为
- GUI 相关改动应尽量限制在 `crates/rsendmail-tauri/`
- 邮件发送行为的修改通常应落在 `rsendmail-core`
