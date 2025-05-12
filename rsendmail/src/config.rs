use clap::Parser;
use log::LevelFilter;

/// A high-performance bulk email sending CLI tool
#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Config {
    /// SMTP服务器地址
    #[arg(long)]
    pub smtp_server: String,

    /// SMTP服务器端口
    #[arg(long, default_value_t = 25)]
    pub port: u16,

    /// 发件人邮箱地址
    #[arg(long)]
    pub from: String,

    /// 收件人邮箱地址
    #[arg(long)]
    pub to: String,

    /// 邮件文件所在目录
    #[arg(long, required_unless_present_any = ["attachment", "attachment_dir"])]
    pub dir: Option<String>,

    /// 邮件文件扩展名
    #[arg(long, default_value = "eml")]
    pub extension: String,

    /// 进程数，auto表示自动设置为CPU核心数，或者指定具体数字
    #[arg(long, default_value = "auto")]
    pub processes: String,

    /// 每个SMTP会话连续发送的邮件数量
    #[arg(long, default_value_t = 1)]
    pub batch_size: usize,

    /// SMTP会话超时时间（秒）
    #[arg(long, default_value_t = 30)]
    pub smtp_timeout: u64,

    /// 日志级别 (error/warn/info/debug/trace)
    #[arg(long, default_value = "info")]
    pub log_level: String,

    /// 是否保留原始邮件头
    #[arg(long, default_value_t = false)]
    pub keep_headers: bool,

    /// 是否匿名化邮箱地址
    #[arg(long, default_value_t = false)]
    pub anonymize_emails: bool,

    /// 邮箱匿名化域名（例如：example.com），匿名化后的邮箱将变为随机字符@domain
    #[arg(long, default_value = "example.com")]
    pub anonymize_domain: String,

    /// 是否使用--from和--to参数修改邮件头中的From和To
    #[arg(long, default_value_t = false)]
    pub modify_headers: bool,

    /// 是否无限循环发送（直到用户中断）
    #[arg(long, default_value_t = false)]
    pub r#loop: bool,

    /// 重复发送次数
    #[arg(long, default_value_t = 1)]
    pub repeat: u32,
    
    /// 循环发送的间隔时间（秒）
    #[arg(long, default_value_t = 1)]
    pub loop_interval: u64,
    
    /// 发送失败后重试的间隔时间（秒）
    #[arg(long, default_value_t = 5)]
    pub retry_interval: u64,

    /// 附件文件路径，用于发送普通文件作为附件
    #[arg(long)]
    pub attachment: Option<String>,
    
    /// 附件目录路径，发送目录下所有文件为单独的邮件
    #[arg(long)]
    pub attachment_dir: Option<String>,

    /// 主题模板，支持变量 {filename}
    #[arg(long)]
    pub subject_template: Option<String>,

    /// 文本内容模板，支持变量 {filename}
    #[arg(long)]
    pub text_template: Option<String>,

    /// HTML内容模板，支持变量 {filename}
    #[arg(long)]
    pub html_template: Option<String>,
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
