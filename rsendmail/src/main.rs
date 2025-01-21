use clap::Parser;
use env_logger::Env;
use log::{info, error};

mod config;
mod mailer;
mod stats;

use crate::{config::Config, mailer::Mailer};
use std::process;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    // 解析命令行参数
    let config = Config::parse();

    // 创建邮件发送器
    let mailer = Mailer::new(config);

    // 发送邮件并获取统计信息
    match mailer.send_all().await {
        Ok(stats) => {
            info!(
                "邮件发送完成！\n\
                总计处理: {} 封邮件\n\
                成功发送: {} 封\n\
                解析失败: {} 封\n\
                发送失败: {} 封\n\
                实际总用时: {:.2}秒\n\
                邮件解析总用时: {:.2}秒，平均每封: {:.2}秒\n\
                邮件发送总用时: {:.2}秒，平均每封: {:.2}秒",
                stats.email_count + stats.parse_errors + stats.send_errors,
                stats.email_count,
                stats.parse_errors,
                stats.send_errors,
                stats.total_duration.as_secs_f64(),
                stats.total_parse_duration().as_secs_f64(),
                stats.average_parse_duration().as_secs_f64(),
                stats.total_send_duration().as_secs_f64(),
                stats.average_send_duration().as_secs_f64(),
            );
        }
        Err(e) => {
            error!("邮件发送失败: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}
