use log::LevelFilter;
use serde::{Deserialize, Serialize};

/// 邮件发送配置（无 CLI 依赖）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// SMTP服务器地址
    pub smtp_server: String,

    /// SMTP服务器端口
    #[serde(default = "default_port")]
    pub port: u16,

    /// 发件人邮箱地址
    pub from: String,

    /// 收件人邮箱地址 (多个地址请用逗号分隔)
    pub to: String,

    /// 邮件文件所在目录
    pub dir: Option<String>,

    /// 邮件文件扩展名
    #[serde(default = "default_extension")]
    pub extension: String,

    /// 进程数，auto表示自动设置为CPU核心数，或者指定具体数字
    #[serde(default = "default_processes")]
    pub processes: String,

    /// 每个SMTP会话连续发送的邮件数量
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,

    /// SMTP会话超时时间（秒）
    #[serde(default = "default_smtp_timeout")]
    pub smtp_timeout: u64,

    /// 日志级别 (error/warn/info/debug/trace)
    #[serde(default = "default_log_level")]
    pub log_level: String,

    /// 是否保留原始邮件头
    #[serde(default)]
    pub keep_headers: bool,

    /// 是否匿名化邮箱地址
    #[serde(default)]
    pub anonymize_emails: bool,

    /// 邮箱匿名化域名（例如：example.com），匿名化后的邮箱将变为随机字符@domain
    #[serde(default = "default_anonymize_domain")]
    pub anonymize_domain: String,

    /// 是否使用--from和--to参数修改邮件头中的From和To
    #[serde(default)]
    pub modify_headers: bool,

    /// 是否无限循环发送（直到用户中断）
    #[serde(default, rename = "loop")]
    pub r#loop: bool,

    /// 重复发送次数
    #[serde(default = "default_repeat")]
    pub repeat: u32,

    /// 循环发送的间隔时间（秒）
    #[serde(default = "default_loop_interval")]
    pub loop_interval: u64,

    /// 发送失败后重试的间隔时间（秒）
    #[serde(default = "default_retry_interval")]
    pub retry_interval: u64,

    /// 附件文件路径，用于发送普通文件作为附件
    pub attachment: Option<String>,

    /// 附件目录路径，发送目录下所有文件为单独的邮件
    pub attachment_dir: Option<String>,

    /// 主题模板，支持变量 {filename}
    pub subject_template: Option<String>,

    /// 文本内容模板，支持变量 {filename}
    pub text_template: Option<String>,

    /// HTML内容模板，支持变量 {filename}
    pub html_template: Option<String>,

    /// 批次内每封邮件发送间隔（毫秒）
    #[serde(default)]
    pub email_send_interval_ms: u64,

    /// 是否使用邮箱账号登录模式（通过用户名和密码验证发送邮件）
    #[serde(default)]
    pub auth_mode: bool,

    /// 邮箱账号用户名（仅在auth_mode=true时需要）
    pub username: Option<String>,

    /// 邮箱账号密码（仅在auth_mode=true时需要）
    pub password: Option<String>,

    /// 使用TLS加密连接 (为了兼容大多数SMTP服务器，当端口是465时将自动启用)
    #[serde(default)]
    pub use_tls: bool,

    /// 是否接受无效的证书
    #[serde(default)]
    pub accept_invalid_certs: bool,

    /// 发送失败的EML文件保存目录
    pub failed_emails_dir: Option<String>,

    /// 日志文件保存路径（如果指定，日志会同时输出到控制台和文件）
    pub log_file: Option<String>,
}

// 默认值函数
fn default_port() -> u16 {
    25
}

fn default_extension() -> String {
    "eml".to_string()
}

fn default_processes() -> String {
    "auto".to_string()
}

fn default_batch_size() -> usize {
    1
}

fn default_smtp_timeout() -> u64 {
    30
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_anonymize_domain() -> String {
    "example.com".to_string()
}

fn default_repeat() -> u32 {
    1
}

fn default_loop_interval() -> u64 {
    1
}

fn default_retry_interval() -> u64 {
    5
}

#[derive(Debug, PartialEq)]
pub enum ProcessMode {
    Auto,
    Fixed(usize),
}

impl Config {
    pub fn get_log_level(&self) -> LevelFilter {
        match self.log_level.to_lowercase().as_str() {
            "error" => LevelFilter::Error,
            "warn" => LevelFilter::Warn,
            "info" => LevelFilter::Info,
            "debug" => LevelFilter::Debug,
            "trace" => LevelFilter::Trace,
            _ => LevelFilter::Info,
        }
    }

    pub fn process_mode(&self) -> ProcessMode {
        if self.processes == "auto" {
            ProcessMode::Auto
        } else {
            match self.processes.parse::<usize>() {
                Ok(n) => ProcessMode::Fixed(n),
                Err(_) => ProcessMode::Auto,
            }
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            smtp_server: String::new(),
            port: default_port(),
            from: String::new(),
            to: String::new(),
            dir: None,
            extension: default_extension(),
            processes: default_processes(),
            batch_size: default_batch_size(),
            smtp_timeout: default_smtp_timeout(),
            log_level: default_log_level(),
            keep_headers: false,
            anonymize_emails: false,
            anonymize_domain: default_anonymize_domain(),
            modify_headers: false,
            r#loop: false,
            repeat: default_repeat(),
            loop_interval: default_loop_interval(),
            retry_interval: default_retry_interval(),
            attachment: None,
            attachment_dir: None,
            subject_template: None,
            text_template: None,
            html_template: None,
            email_send_interval_ms: 0,
            auth_mode: false,
            username: None,
            password: None,
            use_tls: false,
            accept_invalid_certs: false,
            failed_emails_dir: None,
            log_file: None,
        }
    }
}
