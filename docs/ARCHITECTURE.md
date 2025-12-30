# RSendMail 架构设计文档

本文档描述 RSendMail 项目的整体架构设计，包括模块划分、依赖关系、以及 CLI 和 GUI 版本的协同开发方案。

---

## 1. 现状分析

### 1.1 当前项目结构

```
RSendMail/
├── rsendmail/
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs        # 入口点 + 业务编排
│       ├── config.rs      # 配置结构（与 clap 强耦合）
│       ├── mailer.rs      # 核心发送逻辑（1698 行）
│       ├── stats.rs       # 统计模块
│       └── anonymizer.rs  # 邮箱匿名化
├── Dockerfile
└── README.md
```

### 1.2 模块依赖分析

```
                    ┌─────────────┐
                    │   main.rs   │
                    │  (入口点)    │
                    └──────┬──────┘
                           │
            ┌──────────────┼──────────────┐
            │              │              │
            ▼              ▼              ▼
    ┌───────────┐   ┌───────────┐   ┌───────────┐
    │ config.rs │   │ mailer.rs │   │  stats.rs │
    │  (clap)   │   │ (核心)    │   │  (统计)   │
    └───────────┘   └─────┬─────┘   └───────────┘
                          │
                          ▼
                  ┌───────────────┐
                  │ anonymizer.rs │
                  └───────────────┘
```

### 1.3 关键问题识别

| 问题 | 影响 | 解决方案 |
|------|------|----------|
| `Config` 与 `clap` 强耦合 | GUI 无法复用配置结构 | 分离纯数据结构和 CLI 解析 |
| `main.rs` 混合业务编排 | 循环/重试逻辑无法复用 | 抽取为独立的执行器模块 |
| 无 workspace 结构 | CLI 和 GUI 无法共享代码 | 引入 Cargo workspace |
| 日志与 simplelog 耦合 | GUI 需要不同的日志处理 | 使用 `log` trait 抽象 |

---

## 2. 目标架构

### 2.1 Cargo Workspace 结构

```
RSendMail/
├── Cargo.toml                    # Workspace 根配置（虚拟 manifest）
├── Cargo.lock                    # 共享的锁文件
│
├── crates/
│   ├── rsendmail-core/           # 核心库（纯业务逻辑）
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── config.rs         # 纯配置数据结构
│   │       ├── mailer.rs         # 发送逻辑
│   │       ├── executor.rs       # 执行器（循环/重试/取消）
│   │       ├── stats.rs          # 统计
│   │       ├── anonymizer.rs     # 匿名化
│   │       └── error.rs          # 统一错误类型
│   │
│   ├── rsendmail-cli/            # CLI 应用
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs           # CLI 入口
│   │       ├── args.rs           # clap 参数定义
│   │       └── logging.rs        # CLI 日志配置
│   │
│   └── rsendmail-gui/            # GUI 应用（Slint）
│       ├── Cargo.toml
│       ├── build.rs              # Slint 编译配置
│       ├── ui/
│       │   ├── app.slint         # 主应用
│       │   └── components/       # UI 组件
│       └── src/
│           ├── main.rs           # GUI 入口
│           ├── app.rs            # 应用逻辑
│           ├── state.rs          # 状态管理
│           ├── bridge.rs         # Slint <-> Core 桥接
│           └── config_io.rs      # 配置文件读写
│
├── docs/                         # 文档
├── test_data/                    # 测试数据
└── README.md
```

### 2.2 模块职责划分

#### rsendmail-core（核心库）

**职责**：提供与 UI 无关的纯业务逻辑

| 模块 | 职责 | 对外接口 |
|------|------|----------|
| `config` | 配置数据结构定义 | `SendConfig`, `SmtpConfig`, `SendMode` |
| `mailer` | SMTP 发送逻辑 | `Mailer::send_all()`, `Mailer::test_connection()` |
| `executor` | 执行控制（循环/重试/取消） | `Executor::run()`, `ExecutorHandle` |
| `stats` | 统计收集与报告 | `Stats`, `StatsCollector` |
| `anonymizer` | 邮箱匿名化 | `EmailAnonymizer` |
| `error` | 统一错误类型 | `SendError`, `ConfigError` |

**设计原则**：
- 不依赖任何 UI 框架（clap、slint）
- 不直接处理日志输出，使用 `log` crate 的 trait
- 所有公开类型实现 `Clone`, `Debug`, `Serialize`, `Deserialize`

#### rsendmail-cli（CLI 应用）

**职责**：命令行界面和用户交互

| 模块 | 职责 |
|------|------|
| `args` | clap 参数定义，转换为 `core::SendConfig` |
| `logging` | simplelog 初始化和配置 |
| `main` | CLI 入口，调用 `core::Executor` |

#### rsendmail-gui（GUI 应用）

**职责**：图形用户界面

| 模块 | 职责 |
|------|------|
| `app` | 应用主逻辑，事件处理 |
| `state` | UI 状态管理 |
| `bridge` | Slint 回调与 Core 异步操作的桥接 |
| `config_io` | 配置文件（JSON）的保存和加载 |

---

## 3. 核心模块详细设计

### 3.1 配置结构重构

**目标**：将配置数据结构与 clap 解耦

```rust
// crates/rsendmail-core/src/config.rs

use serde::{Deserialize, Serialize};

/// SMTP 服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmtpConfig {
    pub server: String,
    pub port: u16,
    pub timeout_secs: u64,
    pub use_tls: bool,
    pub accept_invalid_certs: bool,
}

impl Default for SmtpConfig {
    fn default() -> Self {
        Self {
            server: String::new(),
            port: 25,
            timeout_secs: 30,
            use_tls: false,
            accept_invalid_certs: false,
        }
    }
}

/// 认证配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub enabled: bool,
    pub username: Option<String>,
    #[serde(skip_serializing)]  // 密码不序列化到文件
    pub password: Option<String>,
}

/// 发送模式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SendMode {
    /// EML 文件批量发送
    EmlBatch {
        dir: String,
        extension: String,
    },
    /// 单文件附件发送
    SingleAttachment {
        file_path: String,
    },
    /// 目录附件发送（每个文件一封邮件）
    DirectoryAttachment {
        dir: String,
    },
}

/// 邮件模板配置
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TemplateConfig {
    pub subject: Option<String>,
    pub text_body: Option<String>,
    pub html_body: Option<String>,
}

/// 邮件头处理模式
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub enum HeaderMode {
    #[default]
    PreserveOriginal,    // 保持原始内容（默认）
    KeepHeaders,         // 保留原始邮件头
    ModifyHeaders,       // 使用 from/to 修改邮件头
}

/// 性能配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub processes: ProcessCount,
    pub batch_size: usize,
    pub send_interval_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessCount {
    Auto,
    Fixed(usize),
}

/// 循环与重试配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopConfig {
    pub infinite_loop: bool,
    pub repeat_count: u32,
    pub loop_interval_secs: u64,
    pub retry_interval_secs: u64,
}

/// 匿名化配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnonymizeConfig {
    pub enabled: bool,
    pub target_domain: String,
}

/// 发送任务完整配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendConfig {
    pub smtp: SmtpConfig,
    pub auth: AuthConfig,
    pub from: String,
    pub to: String,  // 逗号分隔的多个收件人
    pub mode: SendMode,
    pub template: TemplateConfig,
    pub header_mode: HeaderMode,
    pub performance: PerformanceConfig,
    pub loop_config: LoopConfig,
    pub anonymize: AnonymizeConfig,
    pub failed_emails_dir: Option<String>,
}

impl SendConfig {
    /// 验证配置有效性
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.smtp.server.is_empty() {
            return Err(ConfigError::MissingField("smtp.server"));
        }
        if self.from.is_empty() {
            return Err(ConfigError::MissingField("from"));
        }
        if self.to.is_empty() {
            return Err(ConfigError::MissingField("to"));
        }
        // ... 更多验证
        Ok(())
    }

    /// 获取收件人列表
    pub fn recipients(&self) -> Vec<&str> {
        self.to
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect()
    }
}
```

### 3.2 执行器设计

**目标**：封装循环/重试/取消逻辑，提供统一的执行接口

```rust
// crates/rsendmail-core/src/executor.rs

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::mpsc;

/// 执行事件（用于向调用者报告状态）
#[derive(Debug, Clone)]
pub enum ExecutorEvent {
    /// 开始新一轮发送
    IterationStarted { current: u32, total: Option<u32> },
    /// 发送进度更新
    Progress { sent: usize, total: usize, success: usize, failed: usize },
    /// 单封邮件发送结果
    EmailResult { path: String, success: bool, error: Option<String> },
    /// 一轮发送完成
    IterationCompleted { stats: Stats },
    /// 等待下一轮
    WaitingForNextIteration { seconds: u64 },
    /// 全部完成
    AllCompleted { total_stats: Stats },
    /// 发生错误
    Error { message: String },
}

/// 执行器控制句柄
pub struct ExecutorHandle {
    running: Arc<AtomicBool>,
}

impl ExecutorHandle {
    /// 请求停止执行
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    /// 检查是否正在运行
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
}

/// 发送任务执行器
pub struct Executor {
    config: SendConfig,
    running: Arc<AtomicBool>,
}

impl Executor {
    pub fn new(config: SendConfig) -> Self {
        Self {
            config,
            running: Arc::new(AtomicBool::new(true)),
        }
    }

    /// 获取控制句柄
    pub fn handle(&self) -> ExecutorHandle {
        ExecutorHandle {
            running: Arc::clone(&self.running),
        }
    }

    /// 执行发送任务
    /// 返回事件接收器，调用者可以监听执行过程
    pub async fn run(self) -> (mpsc::Receiver<ExecutorEvent>, ExecutorHandle) {
        let (tx, rx) = mpsc::channel(100);
        let handle = self.handle();
        let running = Arc::clone(&self.running);

        tokio::spawn(async move {
            self.run_internal(tx).await;
        });

        (rx, handle)
    }

    async fn run_internal(self, tx: mpsc::Sender<ExecutorEvent>) {
        let mailer = Mailer::new(self.config.clone());
        let mut total_stats = Stats::new();

        let iteration_count = if self.config.loop_config.infinite_loop {
            None
        } else {
            Some(self.config.loop_config.repeat_count)
        };

        let mut current = 1u32;

        loop {
            if !self.running.load(Ordering::SeqCst) {
                break;
            }

            // 发送迭代开始事件
            let _ = tx.send(ExecutorEvent::IterationStarted {
                current,
                total: iteration_count,
            }).await;

            // 执行发送
            match mailer.send_all_with_cancel(
                Arc::clone(&self.running),
                Some(tx.clone()),  // 传递事件发送器
            ).await {
                Ok(stats) => {
                    total_stats.merge(&stats);
                    let _ = tx.send(ExecutorEvent::IterationCompleted {
                        stats
                    }).await;
                }
                Err(e) => {
                    let _ = tx.send(ExecutorEvent::Error {
                        message: e.to_string()
                    }).await;
                }
            }

            // 检查是否继续
            if let Some(total) = iteration_count {
                if current >= total {
                    break;
                }
            }

            // 等待间隔
            if self.running.load(Ordering::SeqCst) {
                let interval = self.config.loop_config.loop_interval_secs;
                let _ = tx.send(ExecutorEvent::WaitingForNextIteration {
                    seconds: interval
                }).await;
                tokio::time::sleep(Duration::from_secs(interval)).await;
            }

            current += 1;
        }

        let _ = tx.send(ExecutorEvent::AllCompleted {
            total_stats
        }).await;
    }
}
```

### 3.3 Mailer 接口调整

```rust
// crates/rsendmail-core/src/mailer.rs

pub struct Mailer {
    config: SendConfig,
}

impl Mailer {
    pub fn new(config: SendConfig) -> Self {
        Self { config }
    }

    /// 测试 SMTP 连接
    pub async fn test_connection(&self) -> Result<ConnectionTestResult, SendError> {
        // 尝试连接并返回详细结果
        todo!()
    }

    /// 扫描待发送文件
    pub fn scan_files(&self) -> Result<Vec<String>, SendError> {
        // 根据 SendMode 扫描文件
        todo!()
    }

    /// 执行发送（带取消支持和事件通知）
    pub async fn send_all_with_cancel(
        &self,
        running: Arc<AtomicBool>,
        event_tx: Option<mpsc::Sender<ExecutorEvent>>,
    ) -> Result<Stats, SendError> {
        // 核心发送逻辑
        todo!()
    }
}

/// 连接测试结果
#[derive(Debug)]
pub struct ConnectionTestResult {
    pub success: bool,
    pub server_greeting: Option<String>,
    pub tls_info: Option<String>,
    pub auth_status: Option<bool>,
    pub latency_ms: u64,
    pub error: Option<String>,
}
```

---

## 4. GUI 与 Core 集成设计

### 4.1 线程模型

```
┌─────────────────────────────────────────────────────────────────┐
│                        GUI 应用进程                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                 主线程 (UI 线程)                         │   │
│  │  ┌─────────────────────────────────────────────────────┐│   │
│  │  │              Slint 事件循环                         ││   │
│  │  │  - UI 渲染                                          ││   │
│  │  │  - 用户输入处理                                      ││   │
│  │  │  - 状态更新                                         ││   │
│  │  └─────────────────────────────────────────────────────┘│   │
│  │                          ↑                               │   │
│  │                          │ invoke_from_event_loop        │   │
│  │                          │                               │   │
│  └──────────────────────────┼───────────────────────────────┘   │
│                             │                                   │
│  ┌──────────────────────────┼───────────────────────────────┐   │
│  │              工作线程 (Tokio Runtime)                    │   │
│  │                          │                               │   │
│  │  ┌─────────────────────────────────────────────────────┐│   │
│  │  │              Executor + Mailer                      ││   │
│  │  │  - SMTP 连接                                        ││   │
│  │  │  - 邮件发送                                          ││   │
│  │  │  - 文件读取                                          ││   │
│  │  └─────────────────────────────────────────────────────┘│   │
│  │                                                         │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 4.2 通信机制

```rust
// crates/rsendmail-gui/src/bridge.rs

use slint::Weak;
use tokio::sync::mpsc;
use rsendmail_core::{Executor, ExecutorEvent, ExecutorHandle, SendConfig};

/// GUI 与后台任务的桥接器
pub struct SendTaskBridge {
    runtime: tokio::runtime::Runtime,
    handle: Option<ExecutorHandle>,
}

impl SendTaskBridge {
    pub fn new() -> Self {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .expect("Failed to create Tokio runtime");

        Self {
            runtime,
            handle: None,
        }
    }

    /// 启动发送任务
    pub fn start_sending<UI: slint::ComponentHandle + 'static>(
        &mut self,
        config: SendConfig,
        ui_weak: Weak<UI>,
        update_fn: impl Fn(&UI, ExecutorEvent) + Send + 'static,
    ) {
        let executor = Executor::new(config);
        let handle = executor.handle();
        self.handle = Some(handle);

        self.runtime.spawn(async move {
            let (mut rx, _handle) = executor.run().await;

            while let Some(event) = rx.recv().await {
                let event_clone = event.clone();
                let ui_weak_clone = ui_weak.clone();
                let update_fn_ref = &update_fn;

                // 安全地更新 UI
                slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_weak_clone.upgrade() {
                        update_fn_ref(&ui, event_clone);
                    }
                }).ok();
            }
        });
    }

    /// 停止发送任务
    pub fn stop_sending(&self) {
        if let Some(ref handle) = self.handle {
            handle.stop();
        }
    }

    /// 检查是否正在运行
    pub fn is_running(&self) -> bool {
        self.handle.as_ref().map(|h| h.is_running()).unwrap_or(false)
    }
}
```

### 4.3 状态管理

```rust
// crates/rsendmail-gui/src/state.rs

use rsendmail_core::{SendConfig, Stats};

/// 应用状态
#[derive(Debug, Clone)]
pub struct AppState {
    /// 当前配置
    pub config: SendConfig,

    /// 发送状态
    pub send_state: SendState,

    /// 统计信息
    pub stats: SendStats,

    /// 日志条目
    pub logs: Vec<LogEntry>,

    /// 配置文件路径
    pub config_file_path: Option<String>,

    /// 配置是否已修改
    pub config_modified: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SendState {
    Idle,
    Preparing,
    Sending,
    Paused,  // 未来扩展
    Stopping,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Default)]
pub struct SendStats {
    pub total_files: usize,
    pub sent_count: usize,
    pub success_count: usize,
    pub fail_count: usize,
    pub current_qps: f64,
    pub elapsed_time: Duration,
    pub current_iteration: u32,
    pub total_iterations: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: DateTime<Local>,
    pub level: LogLevel,
    pub message: String,
}

#[derive(Debug, Clone, Copy)]
pub enum LogLevel {
    Info,
    Warning,
    Error,
}
```

---

## 5. Workspace 配置

### 5.1 根 Cargo.toml

```toml
# RSendMail/Cargo.toml

[workspace]
resolver = "2"
members = [
    "crates/rsendmail-core",
    "crates/rsendmail-cli",
    "crates/rsendmail-gui",
]

# 共享依赖版本
[workspace.dependencies]
# 异步运行时
tokio = { version = "1.35", features = ["full"] }
futures = "0.3"

# 邮件处理
mail-send = "0.5"
mail-parser = "0.10"
mail-builder = "0.3"

# 序列化
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# 错误处理
anyhow = "1.0"
thiserror = "1.0"

# 日志
log = "0.4"

# 文件系统
walkdir = "2.5"

# 其他工具
regex = "1.10"
rand = "0.8"
chrono = "0.4"
num_cpus = "1.16"
infer = "0.15"

# CLI 专用
clap = { version = "4.5", features = ["derive"] }
simplelog = "0.12"
ctrlc = "3.4"

# GUI 专用
slint = "1.9"
rfd = "0.15"  # 文件对话框

[workspace.package]
version = "0.2.0"
edition = "2021"
authors = ["RSendMail Contributors"]
license = "MIT"
repository = "https://github.com/kpassy/RSendMail"
```

### 5.2 rsendmail-core/Cargo.toml

```toml
[package]
name = "rsendmail-core"
description = "Core library for RSendMail - email sending engine"
version.workspace = true
edition.workspace = true
authors.workspace = true

[dependencies]
tokio = { workspace = true }
futures = { workspace = true }
mail-send = { workspace = true }
mail-parser = { workspace = true }
mail-builder = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
anyhow = { workspace = true }
thiserror = { workspace = true }
log = { workspace = true }
walkdir = { workspace = true }
regex = { workspace = true }
rand = { workspace = true }
chrono = { workspace = true }
num_cpus = { workspace = true }
infer = { workspace = true }

[dev-dependencies]
tokio-test = "0.4"
```

### 5.3 rsendmail-cli/Cargo.toml

```toml
[package]
name = "rsendmail-cli"
description = "Command-line interface for RSendMail"
version.workspace = true
edition.workspace = true
authors.workspace = true

[[bin]]
name = "rsendmail"
path = "src/main.rs"

[dependencies]
rsendmail-core = { path = "../rsendmail-core" }
tokio = { workspace = true }
clap = { workspace = true }
simplelog = { workspace = true }
ctrlc = { workspace = true }
log = { workspace = true }
anyhow = { workspace = true }
```

### 5.4 rsendmail-gui/Cargo.toml

```toml
[package]
name = "rsendmail-gui"
description = "Graphical user interface for RSendMail"
version.workspace = true
edition.workspace = true
authors.workspace = true

[[bin]]
name = "rsendmail-gui"
path = "src/main.rs"

[dependencies]
rsendmail-core = { path = "../rsendmail-core" }
tokio = { workspace = true }
slint = { workspace = true }
rfd = { workspace = true }
log = { workspace = true }
anyhow = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
chrono = { workspace = true }

[build-dependencies]
slint-build = "1.9"
```

---

## 6. 迁移计划

### Phase 1: 基础设施搭建

**目标**：建立 workspace 结构，不破坏现有功能

| 步骤 | 工作内容 | 预期产出 |
|------|----------|----------|
| 1.1 | 创建 workspace 目录结构 | `crates/` 目录 |
| 1.2 | 创建 `rsendmail-core` 空壳 | 可编译的空库 |
| 1.3 | 迁移 `anonymizer.rs` 和 `stats.rs` | 首批无依赖模块迁移 |
| 1.4 | 迁移测试确保功能正常 | 通过所有测试 |

### Phase 2: 配置层重构

**目标**：分离配置数据结构与 clap

| 步骤 | 工作内容 | 预期产出 |
|------|----------|----------|
| 2.1 | 创建 `core::config` 纯数据结构 | `SendConfig` 等类型 |
| 2.2 | 在 CLI 中创建 `args.rs` 包装 clap | `CliArgs` 类型 |
| 2.3 | 实现 `CliArgs` -> `SendConfig` 转换 | 转换函数 |
| 2.4 | 验证 CLI 功能完整性 | 回归测试通过 |

### Phase 3: 核心逻辑迁移

**目标**：将 mailer 和执行逻辑迁移到 core

| 步骤 | 工作内容 | 预期产出 |
|------|----------|----------|
| 3.1 | 迁移 `mailer.rs` 到 core | 编译通过 |
| 3.2 | 创建 `executor.rs` 抽取循环逻辑 | `Executor` 类型 |
| 3.3 | 更新 CLI 使用 core 库 | CLI 功能正常 |
| 3.4 | 添加事件通知机制 | `ExecutorEvent` |

### Phase 4: GUI 应用开发

**目标**：创建可用的 GUI 版本

| 步骤 | 工作内容 | 预期产出 |
|------|----------|----------|
| 4.1 | 创建 `rsendmail-gui` 项目 | 项目结构 |
| 4.2 | 实现基础 UI 布局 | 可运行的窗口 |
| 4.3 | 实现 SMTP 配置界面 | 配置可输入 |
| 4.4 | 实现发送功能集成 | 可发送邮件 |
| 4.5 | 实现日志和统计显示 | 实时反馈 |
| 4.6 | 实现配置保存/加载 | 持久化配置 |

### Phase 5: 完善与发布

| 步骤 | 工作内容 | 预期产出 |
|------|----------|----------|
| 5.1 | 统一错误处理 | 友好的错误提示 |
| 5.2 | 完善文档 | README 更新 |
| 5.3 | 添加 CI/CD 构建 | 自动化构建 |
| 5.4 | 发布新版本 | v0.2.0 |

---

## 7. 构建与开发

### 7.1 常用命令

```bash
# 构建所有 crate
cargo build --workspace

# 仅构建 CLI
cargo build -p rsendmail-cli --release

# 仅构建 GUI
cargo build -p rsendmail-gui --release

# 运行 CLI
cargo run -p rsendmail-cli -- --help

# 运行 GUI
cargo run -p rsendmail-gui

# 运行测试
cargo test --workspace

# 运行特定 crate 的测试
cargo test -p rsendmail-core
```

### 7.2 开发工作流

1. **修改核心逻辑**：在 `rsendmail-core` 中修改，然后运行 `cargo test -p rsendmail-core`
2. **修改 CLI**：在 `rsendmail-cli` 中修改，使用 `cargo run -p rsendmail-cli -- <args>` 测试
3. **修改 GUI**：在 `rsendmail-gui` 中修改，使用 `cargo run -p rsendmail-gui` 测试

---

## 8. 设计决策记录

### 8.1 为什么使用 Cargo Workspace？

**替代方案**：
- 单一 crate 通过 feature flags 区分 CLI/GUI
- 完全独立的两个项目

**选择 Workspace 的理由**：
1. 代码复用：核心逻辑只维护一份
2. 统一版本管理：共享 Cargo.lock
3. 编译优化：共享 target 目录，增量编译
4. 清晰的边界：强制模块化

### 8.2 为什么将 Config 与 clap 分离？

**问题**：
- clap 的 `#[derive(Parser)]` 要求特定属性
- GUI 不需要命令行参数
- 配置需要序列化到文件

**解决方案**：
- `core::SendConfig`：纯数据结构，支持 serde
- `cli::CliArgs`：clap 解析，转换为 `SendConfig`
- `gui::ConfigFile`：JSON 读写，转换为 `SendConfig`

### 8.3 为什么使用事件通道而非回调？

**替代方案**：直接传递回调函数

**选择事件通道的理由**：
1. 解耦：Core 不需要知道 UI 的实现
2. 线程安全：mpsc 天然支持跨线程
3. 缓冲：可以处理事件突发
4. 灵活：可以过滤、聚合事件

### 8.4 GUI 的 Tokio 集成策略

**选择方案**：独立线程运行 Tokio Runtime

**理由**：
1. Slint 不是 async 原生的
2. 发送任务是 CPU/IO 密集型，不应阻塞 UI
3. `invoke_from_event_loop` 提供安全的跨线程 UI 更新

---

## 9. 参考资料

- [Cargo Workspaces - The Rust Book](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html)
- [Large Rust Workspaces - matklad](https://matklad.github.io/2021/08/22/large-rust-workspaces.html)
- [How I Use Cargo Workspace - Vivek Shukla](https://vivekshuk.la/tech/2025/use-cargo-workspace-rust/)
- [Slint + async Rust Discussion](https://github.com/slint-ui/slint/discussions/4377)
- [Slint spawn_local Documentation](https://releases.slint.dev/1.5.1/docs/rust/slint/fn.spawn_local)
- [Bridging sync code with Tokio](https://tokio.rs/tokio/topics/bridging)
