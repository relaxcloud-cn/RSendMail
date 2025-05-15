use anyhow::Result;
use log::{error, info, warn};
use mail_parser::MessageParser;
use mail_send::smtp::message::Parameters;
use mail_send::{SmtpClient, SmtpClientBuilder};
use std::fs;
use std::path::Path;
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
use mail_send::mail_builder::MessageBuilder;

pub struct Mailer {
    config: Config,
}

impl Mailer {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    // 处理模板变量替换
    fn process_template(template: &str, filename: &str) -> String {
        template.replace("{filename}", filename)
    }

    // 获取文件名（不含路径）
    fn get_filename(path: &str) -> String {
        Path::new(path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| String::from("未知文件"))
    }

    pub async fn send_all_with_cancel(&self, running: Arc<AtomicBool>) -> Result<Stats> {
        // 检查是否提供了附件目录
        if let Some(attachment_dir) = &self.config.attachment_dir {
            info!("检测到附件目录模式：{}", attachment_dir);
            return self.send_attachment_dir_with_cancel(attachment_dir, running).await;
        }
        
        // 检查是否提供了单个附件
        if let Some(attachment_path) = &self.config.attachment {
            info!("检测到附件模式：{}", attachment_path);
            return self.send_attachment_with_cancel(attachment_path, running).await;
        }
        
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
    
    // 发送附件目录中的所有文件
    async fn send_attachment_dir_with_cancel(
        &self,
        attachment_dir: &str,
        running: Arc<AtomicBool>,
    ) -> Result<Stats> {
        info!("准备发送目录中的所有文件作为附件：{}", attachment_dir);
        let mut stats = Stats::new();
        let start = Instant::now();
        
        // 检查目录是否存在
        let dir_path = Path::new(attachment_dir);
        if !dir_path.exists() || !dir_path.is_dir() {
            error!("附件目录不存在或不是一个目录: {}", attachment_dir);
            return Err(anyhow::anyhow!("附件目录不存在或不是一个目录"));
        }
        
        // 收集目录中的所有文件
        let mut files = Vec::new();
        info!("开始扫描目录中的文件: {}", attachment_dir);
        
        for entry in WalkDir::new(attachment_dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                if let Some(path_str) = entry.path().to_str() {
                    files.push(path_str.to_string());
                }
            }
        }
        
        info!("共找到 {} 个文件用于发送", files.len());
        
        if files.is_empty() {
            info!("目录为空，没有文件可发送");
            stats.total_duration = start.elapsed();
            return Ok(stats);
        }
        
        // 创建SMTP连接
        info!("连接SMTP服务器: {}:{}", self.config.smtp_server, self.config.port);
        let client_result = match timeout(
            Duration::from_secs(self.config.smtp_timeout),
            SmtpClientBuilder::new(self.config.smtp_server.as_str(), self.config.port)
                .connect_plain(),
        ).await {
            Ok(result) => result,
            Err(_) => {
                error!("SMTP连接超时");
                stats.increment_error("SMTP连接超时", attachment_dir);
                return Ok(stats);
            }
        };

        let mut client = match client_result {
            Ok(client) => client,
            Err(e) => {
                error!("SMTP连接失败: {}", e);
                stats.increment_error("SMTP连接失败", attachment_dir);
                return Ok(stats);
            }
        };
        
        // 逐个发送每个文件
        for (file_idx, file_path) in files.iter().enumerate() {
            if !running.load(Ordering::SeqCst) {
                warn!("收到中断信号，正在退出...");
                break;
            }
            
            let send_start = Instant::now();
            
            // 获取文件名用于模板替换
            let filename = Self::get_filename(file_path);
            
            // 准备主题和内容
            let subject = match &self.config.subject_template {
                Some(template) => Self::process_template(template, &filename),
                None => format!("附件: {}", filename),
            };
            
            let text_content = match &self.config.text_template {
                Some(template) => Self::process_template(template, &filename),
                None => format!("请查收附件: {}", filename),
            };

            let html_content = self.config.html_template.as_ref()
                .map(|template| Self::process_template(template, &filename));
            
            // 设置SMTP信封
            let empty_params = Parameters::default();
            if let Err(e) = client.mail_from(self.config.from.as_str(), &empty_params).await {
                error!("设置发件人失败: {}", e);
                stats.increment_error("设置发件人失败", file_path);
                continue;
            }

            if let Err(e) = client.rcpt_to(self.config.to.as_str(), &empty_params).await {
                error!("设置收件人失败: {}", e);
                stats.increment_error("设置收件人失败", file_path);
                continue;
            }
            
            // 读取附件内容
            info!("读取文件: {}", file_path);
            let attachment_content = match fs::read(file_path) {
                Ok(content) => content,
                Err(e) => {
                    error!("读取附件文件失败: {}", e);
                    stats.increment_error("读取附件文件失败", file_path);
                    continue;
                }
            };
            
            // 构建邮件
            let mut builder = MessageBuilder::new()
                .from(("", self.config.from.as_str()))
                .to(self.config.to.as_str())
                .subject(&subject)
                .text_body(&text_content);

            // 添加HTML内容（如果有）
            if let Some(html) = &html_content {
                builder = builder.html_body(html);
            }

            // 添加附件
            // 获取文件的MIME类型
            let mime_type = match infer::get_from_path(file_path) {
                Ok(Some(kind)) => kind.mime_type(),
                _ => "application/octet-stream",
            };

            builder = builder.attachment(mime_type, &filename, &attachment_content[..]);

            // 生成邮件内容
            let mail_content = match builder.write_to_vec() {
                Ok(content) => content,
                Err(e) => {
                    error!("生成邮件内容失败: {}", e);
                    stats.increment_error("生成邮件内容失败", file_path);
                    continue;
                }
            };

            // 发送邮件
            info!("发送附件邮件: {}", filename);
            match timeout(
                Duration::from_secs(self.config.smtp_timeout),
                client.data(&mail_content),
            ).await {
                Ok(result) => match result {
                    Ok(_) => {
                        info!("附件邮件发送成功！ 文件: {}", filename);
                        stats.email_count += 1;
                        stats.send_durations.push(send_start.elapsed());
                    }
                    Err(e) => {
                        error!("邮件发送失败: {}, 文件: {}", e, file_path);
                        stats.increment_error(&format!("邮件发送失败: {}", e), file_path);
                    }
                },
                Err(_) => {
                    error!("邮件发送超时, 文件: {}", file_path);
                    stats.increment_error("邮件发送超时", file_path);
                }
            }

            // Add delay if configured and not the last email in the directory
            if self.config.email_send_interval_ms > 0 && (file_idx < files.len() - 1) {
                info!(
                    "附件目录模式：等待 {}ms 后发送下一个文件 (当前: {}/{})",
                    self.config.email_send_interval_ms,
                    file_idx + 1,
                    files.len()
                );
                let sleep_duration = std::time::Duration::from_millis(self.config.email_send_interval_ms);
                let running_clone_for_sleep = running.clone(); // Clone Arc for the async block

                tokio::select! {
                    biased; // Prioritize checking the shutdown signal
                    _ = async {
                        loop {
                            if !running_clone_for_sleep.load(Ordering::SeqCst) {
                                break;
                            }
                            // Check shutdown signal periodically
                            tokio::time::sleep(Duration::from_millis(100)).await;
                        }
                    } => {
                        warn!("附件目录模式：发送间隔休眠被中断 (文件 {}/{})", file_idx + 1, files.len());
                    }
                    _ = tokio::time::sleep(sleep_duration) => {
                        // Sleep finished normally
                    }
                }
                // Re-check running status after select! block, in case sleep was not interrupted but shutdown was requested during very short sleeps.
                if !running.load(Ordering::SeqCst) {
                    warn!("附件目录模式：收到中断信号，在间隔后退出 (文件 {}/{})", file_idx + 1, files.len());
                    break; // Break the loop for sending directory attachments
                }
            }
        } // End of loop for file_path in &files
        
        // 关闭连接
        let _ = client.quit().await;
        
        stats.total_duration = start.elapsed();
        Ok(stats)
    }

    // 发送附件文件
    async fn send_attachment_with_cancel(
        &self, 
        attachment_path: &str, 
        running: Arc<AtomicBool>
    ) -> Result<Stats> {
        info!("准备发送附件：{}", attachment_path);
        let mut stats = Stats::new();
        let start = Instant::now();

        if !Path::new(attachment_path).exists() {
            error!("附件文件不存在: {}", attachment_path);
            return Err(anyhow::anyhow!("附件文件不存在"));
        }

        // 获取文件名用于模板替换
        let filename = Self::get_filename(attachment_path);
        
        // 准备主题和内容
        let subject = match &self.config.subject_template {
            Some(template) => Self::process_template(template, &filename),
            None => format!("附件: {}", filename),
        };
        
        let text_content = match &self.config.text_template {
            Some(template) => Self::process_template(template, &filename),
            None => format!("请查收附件: {}", filename),
        };

        let html_content = self.config.html_template.as_ref()
            .map(|template| Self::process_template(template, &filename));

        info!("连接SMTP服务器: {}:{}", self.config.smtp_server, self.config.port);
        let client_result = match timeout(
            Duration::from_secs(self.config.smtp_timeout),
            SmtpClientBuilder::new(self.config.smtp_server.as_str(), self.config.port)
                .connect_plain(),
        ).await {
            Ok(result) => result,
            Err(_) => {
                error!("SMTP连接超时");
                stats.increment_error("SMTP连接超时", attachment_path);
                return Ok(stats);
            }
        };

        let mut client = match client_result {
            Ok(client) => client,
            Err(e) => {
                error!("SMTP连接失败: {}", e);
                stats.increment_error("SMTP连接失败", attachment_path);
                return Ok(stats);
            }
        };

        if !running.load(Ordering::SeqCst) {
            warn!("收到中断信号，正在退出...");
            return Ok(stats);
        }

        let send_start = Instant::now();

        // 设置SMTP信封
        let empty_params = Parameters::default();
        if let Err(e) = client.mail_from(self.config.from.as_str(), &empty_params).await {
            error!("设置发件人失败: {}", e);
            stats.increment_error("设置发件人失败", attachment_path);
            return Ok(stats);
        }

        if let Err(e) = client.rcpt_to(self.config.to.as_str(), &empty_params).await {
            error!("设置收件人失败: {}", e);
            stats.increment_error("设置收件人失败", attachment_path);
            return Ok(stats);
        }

        // 读取附件内容
        let attachment_content = match fs::read(attachment_path) {
            Ok(content) => content,
            Err(e) => {
                error!("读取附件文件失败: {}", e);
                stats.increment_error("读取附件文件失败", attachment_path);
                return Ok(stats);
            }
        };

        // 构建邮件
        let mut builder = MessageBuilder::new()
            .from(("", self.config.from.as_str()))
            .to(self.config.to.as_str())
            .subject(&subject)
            .text_body(&text_content);

        // 添加HTML内容（如果有）
        if let Some(html) = &html_content {
            builder = builder.html_body(html);
        }

        // 添加附件
        // 获取文件的MIME类型
        let mime_type = match infer::get_from_path(attachment_path) {
            Ok(Some(kind)) => kind.mime_type(),
            _ => "application/octet-stream",
        };

        builder = builder.attachment(mime_type, &filename, &attachment_content[..]);

        // 生成邮件内容
        let mail_content = match builder.write_to_vec() {
            Ok(content) => content,
            Err(e) => {
                error!("生成邮件内容失败: {}", e);
                stats.increment_error("生成邮件内容失败", attachment_path);
                return Ok(stats);
            }
        };

        // 发送邮件
        match timeout(
            Duration::from_secs(self.config.smtp_timeout),
            client.data(&mail_content),
        ).await {
            Ok(result) => match result {
                Ok(_) => {
                    info!("附件邮件发送成功！");
                    stats.email_count = 1;
                    stats.send_durations.push(send_start.elapsed());
                }
                Err(e) => {
                    error!("邮件发送失败: {}", e);
                    stats.increment_error("邮件发送失败", attachment_path);
                }
            },
            Err(_) => {
                error!("邮件发送超时");
                stats.increment_error("邮件发送超时", attachment_path);
            }
        }

        // 关闭连接
        let _ = client.quit().await;

        stats.total_duration = start.elapsed();
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

                        // DELAY LOGIC for send_fixed_mode_with_cancel (EML sending)
                        // This delay occurs after a batch has been processed by this worker.
                        // If batch_size is 1, this is effectively after every email.
                        // Condition: config.email_send_interval_ms > 0 AND not the last file processed by this worker's CHUNK.
                        // 'j' is the index of the file *just processed* or added to the batch that was just processed.
                        if config.email_send_interval_ms > 0 && j < chunk.len() - 1 {
                            info!(
                                "进程组 {}：批处理完成 (处理到文件索引 {}). 等待 {}ms 后处理下一批/文件.",
                                i + 1, // Process group ID
                                j,     // Index of the last file included in the processed batch within the current chunk
                                config.email_send_interval_ms
                            );
                            let sleep_duration = std::time::Duration::from_millis(config.email_send_interval_ms);
                            let running_clone_for_sleep = running.clone(); // Clone Arc for the async block

                            tokio::select! {
                                biased; // Prioritize checking the shutdown signal
                                _ = async {
                                    loop {
                                        if !running_clone_for_sleep.load(Ordering::SeqCst) {
                                            break;
                                        }
                                        // Check shutdown signal periodically
                                        tokio::time::sleep(Duration::from_millis(100)).await;
                                    }
                                } => {
                                    warn!("进程组 {}：发送间隔休眠被中断 (j={})", i + 1, j);
                                }
                                _ = tokio::time::sleep(sleep_duration) => {
                                    // Sleep finished normally
                                }
                            }
                            // Re-check running status after select! block
                            if !running.load(Ordering::SeqCst) {
                                warn!("进程组 {} 收到中断信号，在间隔后退出 (j={})", i + 1, j);
                                break; // Break the outer loop for this worker
                            }
                        }
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
        
        // 检查dir是否存在
        let dir = match &self.config.dir {
            Some(dir_path) => dir_path,
            None => {
                info!("使用附件模式，跳过邮件文件扫描");
                return Ok(files);
            }
        };
        
        info!("开始扫描目录: {}", dir);

        for entry in WalkDir::new(dir)
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

            // 创建空参数对象
            let empty_params = Parameters::default();

            if config.keep_headers {
                // 使用原始邮件头 - 保留所有原始邮件头
                info!("使用原始邮件头发送邮件");

                // 设置SMTP信封发件人和收件人
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
            } else if config.modify_headers {
                // 修改邮件头中的From和To
                info!("修改邮件头并发送邮件");

                // 使用提取的内容构建新邮件
                let subject = message.subject().unwrap_or("No Subject").to_string();
                let text_content = message.body_text(0).unwrap_or_default().to_string();
                let html_content = message.body_html(0).map(|s| s.to_string());

                // 构建新的邮件内容
                let builder = MessageBuilder::new()
                    .from(("", config.from.as_str()))
                    .to(config.to.as_str())
                    .subject(&subject)
                    .text_body(&text_content);

                let builder = if let Some(html) = &html_content {
                    builder.html_body(html)
                } else {
                    builder
                };

                // 生成邮件内容
                let mail_content = builder.write_to_vec()?;

                // 设置SMTP信封发件人和收件人
                if let Err(e) = client.mail_from(config.from.as_str(), &empty_params).await {
                    return Err(anyhow::anyhow!("设置发件人失败: {}", e));
                }

                if let Err(e) = client.rcpt_to(config.to.as_str(), &empty_params).await {
                    return Err(anyhow::anyhow!("设置收件人失败: {}", e));
                }

                // 发送生成的邮件内容
                match timeout(
                    Duration::from_secs(config.smtp_timeout),
                    client.data(&mail_content),
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
                // 保留原始邮件头，使用SMTP信封传递
                info!("保留原始邮件头并发送邮件");

                // 设置SMTP信封发件人和收件人
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
            }
        }

        Ok(results)
    }
}
