use clap::Parser;

#[derive(Debug, Clone)]
pub enum ProcessMode {
    Auto,
    Fixed(usize),
}

#[derive(Parser, Debug, Clone)]
#[command(author, version, about)]
pub struct Config {
    /// SMTP服务器地址
    #[arg(long)]
    pub smtp_server: String,

    /// SMTP服务器端口
    #[arg(long, default_value_t = 25)]
    pub port: u16,

    /// 发件人邮箱
    #[arg(long)]
    pub from: String,

    /// 收件人邮箱
    #[arg(long)]
    pub to: String,

    /// 邮件文件所在目录
    #[arg(long)]
    pub dir: String,

    /// 邮件文件扩展名
    #[arg(long, default_value = "eml")]
    pub extension: String,

    /// 进程数，auto表示自动设置，或者指定具体数字
    #[arg(long, default_value = "auto")]
    pub processes: String,

    /// 每个SMTP会话连续发送的邮件数量
    #[arg(long, default_value_t = 1)]
    pub batch_size: usize,
}

impl Config {
    pub fn process_mode(&self) -> ProcessMode {
        match self.processes.as_str() {
            "auto" => ProcessMode::Auto,
            n => match n.parse::<usize>() {
                Ok(num) => ProcessMode::Fixed(num),
                Err(_) => ProcessMode::Auto,
            },
        }
    }
}
