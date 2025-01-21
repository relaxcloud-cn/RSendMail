use clap::Parser;

#[derive(Debug, Clone)]
pub enum ProcessMode {
    Auto,
    Fixed(usize),
}

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Config {
    /// SMTP服务器地址
    #[arg(short = 's', long)]
    pub smtp_server: String,

    /// SMTP服务器端口
    #[arg(short = 'P', long, default_value_t = 25)]
    pub port: u16,

    /// 发件人邮箱
    #[arg(short = 'f', long)]
    pub from: String,

    /// 收件人邮箱
    #[arg(short = 't', long)]
    pub to: String,

    /// 邮件文件目录
    #[arg(short = 'd', long)]
    pub dir: String,

    /// 邮件文件扩展名
    #[arg(short = 'e', long, default_value = "eml")]
    pub extension: String,

    /// 并发进程数，使用 "auto" 自动设置
    #[arg(short = 'p', long, default_value = "auto")]
    processes_str: String,
}

impl Config {
    pub fn processes(&self) -> ProcessMode {
        if self.processes_str == "auto" {
            ProcessMode::Auto
        } else {
            match self.processes_str.parse() {
                Ok(n) => ProcessMode::Fixed(n),
                Err(_) => ProcessMode::Auto,
            }
        }
    }
}
