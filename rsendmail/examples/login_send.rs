use mail_send::mail_builder::MessageBuilder;
use mail_send::SmtpClientBuilder;
use std::env;
use std::error::Error;
use std::fmt;
use std::num::ParseIntError;

// 定义我们自己的错误类型
#[derive(Debug)]
enum AppError {
    Parse(ParseIntError),
    Io(std::io::Error),
    Smtp(mail_send::Error),
    Custom(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Parse(e) => write!(f, "解析错误: {}", e),
            AppError::Io(e) => write!(f, "IO错误: {}", e),
            AppError::Smtp(e) => write!(f, "SMTP错误: {}", e),
            AppError::Custom(s) => write!(f, "{}", s),
        }
    }
}

impl Error for AppError {}

impl From<ParseIntError> for AppError {
    fn from(err: ParseIntError) -> Self {
        AppError::Parse(err)
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Io(err)
    }
}

impl From<mail_send::Error> for AppError {
    fn from(err: mail_send::Error) -> Self {
        AppError::Smtp(err)
    }
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    // 解析命令行参数
    let args: Vec<String> = env::args().collect();

    if args.len() < 6 {
        eprintln!(
            "用法: {} <SMTP服务器> <端口> <用户名> <密码> <收件人地址> [<主题>] [<内容>]",
            args[0]
        );
        eprintln!(
            "示例: {} smtp.qq.com 465 your_email@qq.com your_password recipient@example.com",
            args[0]
        );
        return Err(AppError::Custom("参数不足".to_string()));
    }

    let smtp_server = &args[1];
    let port: u16 = args[2].parse()?;
    let username = &args[3];
    let password = &args[4];
    let to_address = &args[5];

    let subject = if args.len() > 6 {
        &args[6]
    } else {
        "测试邮件"
    };
    let body = if args.len() > 7 {
        &args[7]
    } else {
        "这是一封测试邮件，通过RSendMail登录邮箱账号发送模式发送。"
    };

    println!("连接到SMTP服务器: {}:{}", smtp_server, port);
    println!("用户名: {}", username);

    // 创建SMTP客户端
    let mut builder = SmtpClientBuilder::new(smtp_server.as_str(), port);

    // 设置认证信息并使用TLS
    let tls_enabled = port == 465;
    builder = builder
        .credentials((username.as_str(), password.as_str()))
        .implicit_tls(tls_enabled);

    println!("使用TLS: {}", if tls_enabled { "是" } else { "否" });

    // 连接到SMTP服务器
    let mut client = builder
        .connect()
        .await
        .map_err(|e| AppError::Custom(format!("连接或认证失败: {}", e)))?;

    println!("成功连接到SMTP服务器并通过认证");

    // 构建邮件
    let message = MessageBuilder::new()
        .from(("RSendMail Test", username.as_str()))
        .to(to_address.as_str())
        .subject(subject)
        .text_body(body);

    // 发送邮件
    client
        .send(message)
        .await
        .map_err(|e| AppError::Custom(format!("邮件发送失败: {}", e)))?;

    println!("邮件发送成功！");

    // 关闭连接
    let _ = client.quit().await;

    Ok(())
}
