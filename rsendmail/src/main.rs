use clap::Parser;
use env_logger::Env;
use log::{error, info, warn};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

mod anonymizer;
mod config;
mod mailer;
mod stats;

use crate::{config::Config, mailer::Mailer, stats::Stats};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 解析命令行参数
    let config = Config::parse();

    // 初始化日志
    env_logger::Builder::from_env(Env::default())
        .filter_level(config.get_log_level())
        .format_timestamp_millis()
        .init();

    // 创建一个原子布尔值用于控制程序退出
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    // 设置Ctrl+C处理器
    ctrlc::set_handler(move || {
        warn!("接收到中断信号，正在优雅退出...");
        r.store(false, Ordering::SeqCst);
    })?;

    // 创建邮件发送器
    let mailer = Mailer::new(config.clone());

    // 设置循环次数
    let mut iteration_count = if config.r#loop {
        // 如果是循环模式，将设置一个非常大的数，实际上会一直运行直到用户中断
        u32::MAX
    } else {
        config.repeat
    };

    // 用于跟踪总体统计信息
    let mut total_stats = Stats::new();
    let total_start_time = Instant::now();
    let mut successful_iterations = 0;

    // 循环发送
    let mut current_iteration = 1;
    while iteration_count > 0 && running.load(Ordering::SeqCst) {
        info!(
            "开始第 {}/{} 轮发送",
            current_iteration,
            if config.r#loop {
                String::from("无限")
            } else {
                config.repeat.to_string()
            }
        );

        // 发送邮件并获取统计信息
        match mailer.send_all_with_cancel(running.clone()).await {
            Ok(stats) => {
                successful_iterations += 1;

                // 累加统计信息
                total_stats.email_count += stats.email_count;

                // 使用迭代器的方式添加耗时数据，避免移动所有权
                for duration in &stats.parse_durations {
                    total_stats.parse_durations.push(*duration);
                }
                for duration in &stats.send_durations {
                    total_stats.send_durations.push(*duration);
                }

                total_stats.parse_errors += stats.parse_errors;
                total_stats.send_errors += stats.send_errors;

                // 累加错误详情
                for (error_type, count) in &stats.error_details {
                    let entry = total_stats
                        .error_details
                        .entry(error_type.clone())
                        .or_insert(0);
                    *entry += count;
                }

                // 累加失败文件
                for (error_type, files) in &stats.failed_files {
                    total_stats
                        .failed_files
                        .entry(error_type.clone())
                        .or_insert_with(Vec::new)
                        .extend(files.clone());
                }

                info!("第 {} 轮发送完成！", current_iteration);
                info!("{}", stats);

                // 如果设置了循环间隔，且不是最后一次循环，则等待一段时间
                if iteration_count > 1 && running.load(Ordering::SeqCst) {
                    info!("等待{}秒后开始下一轮发送...", config.loop_interval);
                    tokio::time::sleep(Duration::from_secs(config.loop_interval)).await;
                }
            }
            Err(e) => {
                error!("第 {} 轮发送失败: {}", current_iteration, e);
                // 如果是循环模式且用户没有中断，则继续下一次循环
                if !config.r#loop || !running.load(Ordering::SeqCst) {
                    return Err(e);
                }
                // 否则等待后重试
                info!("等待{}秒后重试...", config.retry_interval);
                tokio::time::sleep(Duration::from_secs(config.retry_interval)).await;
            }
        }

        current_iteration += 1;
        iteration_count -= 1;
    }

    // 显示总体统计信息
    // 修改条件：只要有成功的迭代就显示统计
    if successful_iterations > 0 {
        total_stats.total_duration = total_start_time.elapsed();
        info!("所有发送轮次完成！总计 {} 轮", successful_iterations);
        info!("总体统计信息:");
        info!("{}", total_stats);
    }

    Ok(())
}
