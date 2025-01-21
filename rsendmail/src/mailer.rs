use crate::{config::Config, stats::Stats};
use anyhow::Result;
use log::{info, error, warn};
use mail_builder::MessageBuilder;
use mail_parser::MessageParser;
use mail_send::SmtpClientBuilder;
use std::{
    fs,
    time::{Duration, Instant},
    sync::{atomic::{AtomicBool, Ordering}, Arc},
};
use tokio::{task, time::timeout};
use walkdir::WalkDir;

pub struct Mailer {
    config: Config,
}

impl Mailer {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub async fn send_all_with_cancel(&self, running: Arc<AtomicBool>) -> Result<Stats> {
        let mut stats = Stats::new();
        info!("开始收集邮件文件...");
        let files = self.collect_email_files()?;
        
        info!("找到 {} 个邮件文件", files.len());
        if files.is_empty() {
            info!("没有找到任何邮件文件，请检查目录 {} 是否包含 .{} 文件", self.config.dir, self.config.extension);
            return Ok(stats);
        }
        
        info!("使用SMTP服务器: {}:{}", self.config.smtp_server, self.config.port);
        info!("发件人: {}", self.config.from);
        info!("收件人: {}", self.config.to);
        
        // 先测试SMTP连接
        info!("测试SMTP服务器连接...");
        match Self::test_smtp_connection(&self.config).await {
            Ok(_) => info!("SMTP服务器连接测试成功！"),
            Err(e) => {
                error!("SMTP服务器连接测试失败: {}", e);
                return Err(anyhow::anyhow!("SMTP服务器连接失败，请检查服务器地址和端口是否正确"));
            }
        }
        
        match self.config.processes() {
            crate::config::ProcessMode::Auto => {
                info!("使用自动并发模式");
                self.send_auto_mode_with_cancel(files, &mut stats, running).await?;
            }
            crate::config::ProcessMode::Fixed(n) => {
                info!("使用固定并发模式: {} 个进程", n);
                self.send_fixed_mode_with_cancel(files, n, &mut stats, running).await?;
            }
        }
        
        Ok(stats)
    }

    async fn send_auto_mode_with_cancel(&self, files: Vec<String>, stats: &mut Stats, running: Arc<AtomicBool>) -> Result<()> {
        let start = Instant::now();
        let num_cpus = num_cpus::get();
        let chunk_size = (files.len() + num_cpus - 1) / num_cpus;
        
        let mut handles = vec![];
        for (i, chunk) in files.chunks(chunk_size).enumerate() {
            let chunk = chunk.to_vec();
            let config = self.config.clone();
            let running = running.clone();
            
            let handle = task::spawn(async move {
                let mut group_stats = (0, Duration::default(), Duration::default());
                for (j, file) in chunk.iter().enumerate() {
                    if !running.load(Ordering::SeqCst) {
                        warn!("进程组 {} 收到中断信号，正在退出...", i + 1);
                        break;
                    }
                    
                    info!("进程组 {} 开始发送文件 {}/{}: {}", i + 1, j + 1, chunk.len(), file);
                    match Self::send_single_email(&config, file).await {
                        Ok((parse_duration, send_duration)) => {
                            info!("进程组 {} 文件 {} 发送成功，用时: {:.2}秒", 
                                i + 1, j + 1, send_duration.as_secs_f64());
                            group_stats.0 += 1;
                            group_stats.1 += parse_duration;
                            group_stats.2 += send_duration;
                        }
                        Err(e) => {
                            error!("进程组 {} 文件 {} 发送失败: {}", i + 1, j + 1, e);
                        }
                    }
                }
                info!("进程组 {} 完成", i + 1);
                group_stats
            });
            handles.push(handle);
        }

        let mut total_sent = 0;
        let mut total_parse_duration = Duration::default();
        let mut total_send_duration = Duration::default();

        for handle in handles {
            if let Ok((sent, parse_duration, send_duration)) = handle.await {
                total_sent += sent;
                total_parse_duration += parse_duration;
                total_send_duration += send_duration;
            }
        }

        stats.email_count = total_sent;
        stats.parse_durations = vec![total_parse_duration];
        stats.send_durations = vec![total_send_duration];
        stats.total_duration = start.elapsed();

        Ok(())
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
                let mut group_stats = (0, Duration::default(), Duration::default());
                for (j, file) in chunk.iter().enumerate() {
                    if !running.load(Ordering::SeqCst) {
                        warn!("进程组 {} 收到中断信号，正在退出...", i + 1);
                        break;
                    }
                    
                    info!("进程组 {} 开始发送文件 {}/{}: {}", i + 1, j + 1, chunk.len(), file);
                    match Self::send_single_email(&config, file).await {
                        Ok((parse_duration, send_duration)) => {
                            info!("进程组 {} 文件 {} 发送成功，用时: {:.2}秒", 
                                i + 1, j + 1, send_duration.as_secs_f64());
                            group_stats.0 += 1;
                            group_stats.1 += parse_duration;
                            group_stats.2 += send_duration;
                        }
                        Err(e) => {
                            error!("进程组 {} 文件 {} 发送失败: {}", i + 1, j + 1, e);
                        }
                    }
                }
                info!("进程组 {} 完成", i + 1);
                group_stats
            });
            handles.push(handle);
        }

        let mut total_sent = 0;
        let mut total_parse_duration = Duration::default();
        let mut total_send_duration = Duration::default();

        for handle in handles {
            if let Ok((sent, parse_duration, send_duration)) = handle.await {
                total_sent += sent;
                total_parse_duration += parse_duration;
                total_send_duration += send_duration;
            }
        }

        stats.email_count = total_sent;
        stats.parse_durations = vec![total_parse_duration];
        stats.send_durations = vec![total_send_duration];
        stats.total_duration = start.elapsed();

        Ok(())
    }

    async fn test_smtp_connection(config: &Config) -> Result<()> {
        match timeout(Duration::from_secs(10), async {
            let mut client = SmtpClientBuilder::new(config.smtp_server.as_str(), config.port)
                .implicit_tls(false)
                .allow_invalid_certs()  // 允许自签名证书
                .connect()
                .await?;
            
            // 尝试发送NOOP命令
            client.noop().await?;
            Ok::<(), anyhow::Error>(())
        }).await {
            Ok(result) => result?,
            Err(_) => return Err(anyhow::anyhow!("SMTP服务器连接超时")),
        }
        Ok(())
    }

    fn collect_email_files(&self) -> Result<Vec<String>> {
        let mut files = Vec::new();
        info!("开始扫描目录: {}", self.config.dir);
        for entry in WalkDir::new(&self.config.dir) {
            match entry {
                Ok(entry) => {
                    if entry.file_type().is_file() {
                        if let Some(ext) = entry.path().extension() {
                            if ext.to_string_lossy() == self.config.extension {
                                if let Some(path) = entry.path().to_str() {
                                    info!("找到邮件文件: {}", path);
                                    files.push(path.to_string());
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("扫描目录时出错: {}", e);
                }
            }
        }
        Ok(files)
    }

    async fn send_single_email(config: &Config, file_path: &str) -> Result<(Duration, Duration)> {
        info!("开始读取文件: {}", file_path);
        let parse_start = Instant::now();
        let content = fs::read_to_string(file_path)?;
        
        info!("解析邮件内容");
        let message = match MessageParser::default().parse(content.as_bytes()) {
            Some(msg) => msg,
            None => {
                error!("无法解析邮件文件: {}", file_path);
                return Err(anyhow::anyhow!("无法解析邮件文件"));
            }
        };
        let parse_duration = parse_start.elapsed();

        let subject = message.subject().unwrap_or("No Subject").to_string();
        let text_content = message.body_text(0).unwrap_or_default().to_string();
        let html_content = message.body_html(0).map(|s| s.to_string());

        info!("连接SMTP服务器: {}:{}", config.smtp_server, config.port);
        let send_start = Instant::now();
        let mut client = match timeout(Duration::from_secs(10), 
            SmtpClientBuilder::new(config.smtp_server.as_str(), config.port)
                .implicit_tls(false)
                .allow_invalid_certs()
                .connect()
        ).await {
            Ok(result) => match result {
                Ok(client) => client,
                Err(e) => return Err(anyhow::anyhow!("SMTP连接失败: {}", e)),
            },
            Err(_) => return Err(anyhow::anyhow!("SMTP连接超时")),
        };

        info!("构建邮件: 主题「{}」", subject);
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

        info!("发送邮件...");
        match timeout(Duration::from_secs(30), client.send(builder)).await {
            Ok(result) => match result {
                Ok(_) => {
                    info!("邮件发送成功！");
                    Ok((parse_duration, send_start.elapsed()))
                },
                Err(e) => Err(anyhow::anyhow!("邮件发送失败: {}", e)),
            },
            Err(_) => Err(anyhow::anyhow!("邮件发送超时")),
        }
    }
}
