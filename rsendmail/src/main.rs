use clap::Parser;
use env_logger::Env;
use log::{info, error, warn};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

mod config;
mod mailer;
mod stats;

use crate::{config::Config, mailer::Mailer};
use std::process;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    // 创建一个原子布尔值用于控制程序退出
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    // 设置Ctrl+C处理器
    ctrlc::set_handler(move || {
        warn!("接收到中断信号，正在优雅退出...");
        r.store(false, Ordering::SeqCst);
    })?;

    // 解析命令行参数
    let config = Config::parse();

    // 创建邮件发送器
    let mailer = Mailer::new(config);

    // 发送邮件并获取统计信息
    match mailer.send_all_with_cancel(running).await {
        Ok(stats) => {
            info!("邮件发送完成！");
            info!("{}", stats);
        }
        Err(e) => {
            error!("邮件发送失败: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}
