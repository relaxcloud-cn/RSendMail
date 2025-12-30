use clap::Parser;
use rsendmail_core::Config;

/// A high-performance bulk email sending CLI tool
#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct CliArgs {
    /// SMTP服务器地址
    #[arg(long)]
    pub smtp_server: String,

    /// SMTP服务器端口
    #[arg(long, default_value_t = 25)]
    pub port: u16,

    /// 发件人邮箱地址
    #[arg(long)]
    pub from: String,

    /// 收件人邮箱地址 (多个地址请用逗号分隔)
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

    #[arg(
        long,
        value_parser,
        default_value_t = 0,
        help = "Interval in milliseconds between sending each email in a batch."
    )]
    pub email_send_interval_ms: u64,

    /// 是否使用邮箱账号登录模式（通过用户名和密码验证发送邮件）
    #[arg(long, default_value_t = false)]
    pub auth_mode: bool,

    /// 邮箱账号用户名（仅在auth_mode=true时需要）
    #[arg(long)]
    pub username: Option<String>,

    /// 邮箱账号密码（仅在auth_mode=true时需要）
    #[arg(long)]
    pub password: Option<String>,

    /// 使用TLS加密连接 (为了兼容大多数SMTP服务器，当端口是465时将自动启用)
    #[arg(long, default_value_t = false)]
    pub use_tls: bool,

    /// 是否接受无效的证书
    #[arg(long, default_value_t = false)]
    pub accept_invalid_certs: bool,

    /// 发送失败的EML文件保存目录
    #[arg(long)]
    pub failed_emails_dir: Option<String>,

    /// 日志文件保存路径（如果指定，日志会同时输出到控制台和文件）
    #[arg(long)]
    pub log_file: Option<String>,
}

impl CliArgs {
    /// 转换为 core 库的配置结构
    pub fn into_config(self) -> Config {
        Config {
            smtp_server: self.smtp_server,
            port: self.port,
            from: self.from,
            to: self.to,
            dir: self.dir,
            extension: self.extension,
            processes: self.processes,
            batch_size: self.batch_size,
            smtp_timeout: self.smtp_timeout,
            log_level: self.log_level,
            keep_headers: self.keep_headers,
            anonymize_emails: self.anonymize_emails,
            anonymize_domain: self.anonymize_domain,
            modify_headers: self.modify_headers,
            r#loop: self.r#loop,
            repeat: self.repeat,
            loop_interval: self.loop_interval,
            retry_interval: self.retry_interval,
            attachment: self.attachment,
            attachment_dir: self.attachment_dir,
            subject_template: self.subject_template,
            text_template: self.text_template,
            html_template: self.html_template,
            email_send_interval_ms: self.email_send_interval_ms,
            auth_mode: self.auth_mode,
            username: self.username,
            password: self.password,
            use_tls: self.use_tls,
            accept_invalid_certs: self.accept_invalid_certs,
            failed_emails_dir: self.failed_emails_dir,
            log_file: self.log_file,
        }
    }
}
