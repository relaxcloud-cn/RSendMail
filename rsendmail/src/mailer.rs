use anyhow::Result;
use log::{error, info, warn};
use mail_parser::MessageParser;
use mail_send::smtp::message::Parameters;
use mail_send::{mail_builder::MessageBuilder, SmtpClient, SmtpClientBuilder};
use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::task;
use tokio::time::timeout;
use walkdir::WalkDir;

use crate::anonymizer::EmailAnonymizer;
use crate::config::Config;
use crate::stats::Stats;

pub struct Mailer {
    config: Config,
}

impl Mailer {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub async fn send_all_with_cancel(&self, running: Arc<AtomicBool>) -> Result<Stats> {
        let files = self.collect_email_files()?;
        let mut stats = Stats::new();

        match self.config.process_mode() {
            crate::config::ProcessMode::Auto => {
                let num_processes = num_cpus::get();
                info!("自动设置进程数为: {}", num_processes);
                self.send_fixed_mode_with_cancel(files, num_processes, &mut stats, running)
                    .await?;
            }
            crate::config::ProcessMode::Fixed(n) => {
                info!("使用指定的进程数: {}", n);
                self.send_fixed_mode_with_cancel(files, n, &mut stats, running)
                    .await?;
            }
        }

        Ok(stats)
    }

    async fn send_fixed_mode_with_cancel(
        &self,
        files: Vec<String>,
        num_processes: usize,
        stats: &mut Stats,
        running: Arc<AtomicBool>,
    ) -> Result<()> {
        let start = Instant::now();
        let chunk_size = (files.len() + num_processes - 1) / num_processes;

        let mut handles = vec![];
        for (i, chunk) in files.chunks(chunk_size).enumerate() {
            let chunk = chunk.to_vec();
            let config = self.config.clone();
            let running = running.clone();

            let handle = task::spawn(async move {
                let mut group_stats = (0, Vec::new(), Vec::new(), Vec::new());
                let mut current_batch = Vec::new();
                let mut client_opt: Option<SmtpClient<tokio::net::TcpStream>> = None;

                for (j, file) in chunk.iter().enumerate() {
                    if !running.load(Ordering::SeqCst) {
                        warn!("进程组 {} 收到中断信号，正在退出...", i + 1);
                        break;
                    }

                    current_batch.push(file.clone());

                    if current_batch.len() >= config.batch_size || j == chunk.len() - 1 {
                        info!(
                            "进程组 {} 开始发送第 {}/{} 批，包含 {} 封邮件",
                            i + 1,
                            j / config.batch_size + 1,
                            (chunk.len() + config.batch_size - 1) / config.batch_size,
                            current_batch.len()
                        );

                        // 如果没有活动连接，创建一个新的
                        if client_opt.is_none() {
                            info!("连接SMTP服务器: {}:{}", config.smtp_server, config.port);
                            let client_result = match timeout(
                                Duration::from_secs(config.smtp_timeout),
                                SmtpClientBuilder::new(config.smtp_server.as_str(), config.port)
                                    .connect_plain(),
                            )
                            .await
                            {
                                Ok(result) => result,
                                Err(_) => {
                                    error!("SMTP连接超时");
                                    for file in &current_batch {
                                        group_stats
                                            .3
                                            .push(("SMTP连接超时".to_string(), file.clone()));
                                    }
                                    current_batch.clear();
                                    continue;
                                }
                            };

                            match client_result {
                                Ok(client) => {
                                    client_opt = Some(client);
                                }
                                Err(e) => {
                                    error!("SMTP连接失败: {}", e);
                                    for file in &current_batch {
                                        group_stats
                                            .3
                                            .push(("SMTP连接失败".to_string(), file.clone()));
                                    }
                                    current_batch.clear();
                                    continue;
                                }
                            }
                        }

                        if let Some(ref mut client) = client_opt {
                            match Self::send_batch_emails(&config, &current_batch, client).await {
                                Ok(results) => {
                                    for (parse_duration, send_duration) in results {
                                        group_stats.0 += 1;
                                        group_stats.1.push(parse_duration);
                                        group_stats.2.push(send_duration);
                                    }
                                }
                                Err(e) => {
                                    error!("批量发送失败: {}", e);
                                    for file in &current_batch {
                                        group_stats.3.push((e.to_string(), file.clone()));
                                    }
                                    // 连接可能已经损坏，重置连接
                                    client_opt = None;
                                }
                            }
                        }

                        current_batch.clear();
                    }
                }

                // 关闭SMTP连接
                if let Some(client) = client_opt {
                    let _ = client.quit().await;
                }

                info!("进程组 {} 完成", i + 1);
                group_stats
            });
            handles.push(handle);
        }

        let mut total_sent = 0;
        for handle in handles {
            if let Ok((sent, parse_durations, send_durations, errors)) = handle.await {
                total_sent += sent;
                stats.parse_durations.extend(parse_durations);
                stats.send_durations.extend(send_durations);
                for (error_type, file_path) in errors {
                    stats.increment_error(&error_type, &file_path);
                }
            }
        }

        stats.email_count = total_sent;
        stats.total_duration = start.elapsed();

        Ok(())
    }

    fn collect_email_files(&self) -> Result<Vec<String>> {
        let mut files = Vec::new();
        info!("开始扫描目录: {}", self.config.dir);

        for entry in WalkDir::new(&self.config.dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                if let Some(ext) = entry.path().extension() {
                    if ext.to_string_lossy() == self.config.extension {
                        if let Some(path_str) = entry.path().to_str() {
                            files.push(path_str.to_string());
                        }
                    }
                }
            }
        }

        info!("共找到 {} 个邮件文件", files.len());
        Ok(files)
    }

    async fn send_batch_emails<T: AsyncRead + AsyncWrite + Unpin + Send>(
        config: &Config,
        files: &[String],
        client: &mut SmtpClient<T>,
    ) -> Result<Vec<(Duration, Duration)>> {
        let mut results = Vec::new();
        // 如果启用邮箱匿名化，创建匿名器
        let mut anonymizer = if config.anonymize_emails {
            Some(EmailAnonymizer::new(&config.anonymize_domain))
        } else {
            None
        };

        for file_path in files {
            info!("读取文件: {}", file_path);
            let parse_start = Instant::now();
            let mut content = fs::read(file_path)?;

            // 如果启用了邮箱匿名化，处理内容
            if let Some(anonymizer) = anonymizer.as_mut() {
                info!("对邮件内容进行邮箱匿名化处理");
                content = anonymizer.anonymize_binary(&content);
            }

            info!("解析邮件内容");
            let message = match MessageParser::default().parse(&content) {
                Some(msg) => msg,
                None => {
                    error!("无法解析邮件文件: {}", file_path);
                    return Err(anyhow::anyhow!("无法解析邮件文件"));
                }
            };
            let parse_duration = parse_start.elapsed();

            let send_start = Instant::now();

            if config.keep_headers {
                // 使用原始邮件头
                info!("使用原始邮件头发送邮件");

                // 创建空参数对象
                let empty_params = Parameters::default();

                // 设置信封发件人和收件人
                if let Err(e) = client.mail_from(config.from.as_str(), &empty_params).await {
                    return Err(anyhow::anyhow!("设置发件人失败: {}", e));
                }

                if let Err(e) = client.rcpt_to(config.to.as_str(), &empty_params).await {
                    return Err(anyhow::anyhow!("设置收件人失败: {}", e));
                }

                // 发送原始邮件内容
                match timeout(
                    Duration::from_secs(config.smtp_timeout),
                    client.data(&content),
                )
                .await
                {
                    Ok(result) => match result {
                        Ok(_) => {
                            info!("邮件发送成功！");
                            results.push((parse_duration, send_start.elapsed()));
                        }
                        Err(e) => return Err(anyhow::anyhow!("邮件发送失败: {}", e)),
                    },
                    Err(_) => return Err(anyhow::anyhow!("邮件发送超时")),
                }
            } else {
                // 使用提取的内容构建新邮件
                let subject = message.subject().unwrap_or("No Subject").to_string();
                let text_content = message.body_text(0).unwrap_or_default().to_string();
                let html_content = message.body_html(0).map(|s| s.to_string());

                info!("构建并发送邮件: 主题「{}」", subject);
                let builder = {
                    let mut b = MessageBuilder::new()
                        .from(("", config.from.as_str()))
                        .to(config.to.as_str())
                        .subject(&subject)
                        .text_body(&text_content);

                    if let Some(html) = &html_content {
                        b = b.html_body(html);
                    }
                    b
                };

                match timeout(
                    Duration::from_secs(config.smtp_timeout),
                    client.send(builder),
                )
                .await
                {
                    Ok(result) => match result {
                        Ok(_) => {
                            info!("邮件发送成功！");
                            results.push((parse_duration, send_start.elapsed()));
                        }
                        Err(e) => return Err(anyhow::anyhow!("邮件发送失败: {}", e)),
                    },
                    Err(_) => return Err(anyhow::anyhow!("邮件发送超时")),
                }
            }
        }

        Ok(results)
    }
}
