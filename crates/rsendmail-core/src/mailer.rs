use anyhow::Result;
use log::{error, info, warn};
use mail_parser::MessageParser;
use mail_send::smtp::message::Parameters;
use mail_send::{SmtpClient, SmtpClientBuilder};
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::task;
use tokio::time::timeout;
use walkdir::WalkDir;

use crate::anonymizer::EmailAnonymizer;
use crate::config::Config;
use crate::stats::Stats;
use mail_send::mail_builder::MessageBuilder;

// Type alias for group statistics to reduce complexity
type GroupStats = (usize, Vec<Duration>, Vec<Duration>, Vec<(String, String)>);

// Structure to hold email content parameters
struct EmailContent<'a> {
    filename: &'a str,
    subject: &'a str,
    text_content: &'a str,
    html_content: &'a Option<String>,
}

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

    // 保存发送失败的EML文件到指定目录
    fn save_failed_email(config: &Config, source_path: &str) {
        if let Some(ref failed_dir) = config.failed_emails_dir {
            let failed_dir_path = Path::new(failed_dir);

            // 创建目录（如果不存在）
            if let Err(e) = fs::create_dir_all(failed_dir_path) {
                error!("创建失败邮件保存目录失败 {}: {}", failed_dir, e);
                return;
            }

            // 获取源文件名
            let source_file_path = Path::new(source_path);
            let original_filename = source_file_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown.eml");

            // 生成唯一的目标文件名（添加时间戳避免覆盖）
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or(Duration::from_secs(0))
                .as_millis();

            let dest_filename = if original_filename.contains('.') {
                let parts: Vec<&str> = original_filename.rsplitn(2, '.').collect();
                if parts.len() == 2 {
                    format!("{}_{}.{}", parts[1], timestamp, parts[0])
                } else {
                    format!("{}_{}", original_filename, timestamp)
                }
            } else {
                format!("{}_{}", original_filename, timestamp)
            };

            let dest_path = failed_dir_path.join(&dest_filename);

            // 复制文件
            match fs::copy(source_path, &dest_path) {
                Ok(_) => {
                    info!("已保存失败邮件: {} -> {}", source_path, dest_path.display());
                }
                Err(e) => {
                    error!(
                        "保存失败邮件时出错 {} -> {}: {}",
                        source_path,
                        dest_path.display(),
                        e
                    );
                }
            }
        }
    }

    pub async fn send_all_with_cancel(&self, running: Arc<AtomicBool>) -> Result<Stats> {
        if let Some(attachment_dir) = &self.config.attachment_dir {
            info!("检测到附件目录模式：{}", attachment_dir);
            return self
                .send_attachment_dir_with_cancel(attachment_dir, running)
                .await;
        }

        if let Some(attachment_path) = &self.config.attachment {
            info!("检测到附件模式：{}", attachment_path);
            return self
                .send_attachment_with_cancel(attachment_path, running)
                .await;
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

    async fn send_attachment_dir_with_cancel(
        &self,
        attachment_dir: &str,
        running: Arc<AtomicBool>,
    ) -> Result<Stats> {
        info!("准备发送目录中的所有文件作为附件：{}", attachment_dir);
        let mut stats = Stats::new();
        let start = Instant::now();

        let dir_path = Path::new(attachment_dir);
        if !dir_path.exists() || !dir_path.is_dir() {
            error!("附件目录不存在或不是一个目录: {}", attachment_dir);
            return Err(anyhow::anyhow!("附件目录不存在或不是一个目录"));
        }

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

        // For send_attachment_dir_with_cancel, assuming non-authenticated, plain connection for simplicity for now.
        // If auth or TLS is needed here, logic similar to send_attachment_with_cancel is required.
        info!(
            "连接SMTP服务器: {}:{}",
            self.config.smtp_server, self.config.port
        );
        let client_builder =
            SmtpClientBuilder::new(self.config.smtp_server.as_str(), self.config.port);

        // Simplified: if TLS is configured for attachment_dir, it would be complex without auth details.
        // Sticking to plain connection for this mode as per original simpler logic.
        // If self.config.use_tls or self.config.port == 465, this mode might not work as expected without full TLS/auth setup.
        // For now, we assume connect_plain is the intended path for this specific function (send_attachment_dir)

        let client_result = match timeout(
            Duration::from_secs(self.config.smtp_timeout),
            client_builder.connect_plain(),
        )
        .await
        {
            Ok(result) => result,
            Err(_) => {
                error!("SMTP连接超时 (附件目录模式)");
                stats.increment_error("SMTP连接超时 (附件目录模式)", attachment_dir);
                return Ok(stats); // Return stats with error
            }
        };

        let mut client = match client_result {
            Ok(client) => client,
            Err(e) => {
                error!("SMTP连接失败 (附件目录模式): {}", e);
                stats.increment_error("SMTP连接失败 (附件目录模式)", attachment_dir);
                return Ok(stats); // Return stats with error
            }
        };

        for (file_idx, file_path) in files.iter().enumerate() {
            if !running.load(Ordering::SeqCst) {
                warn!("收到中断信号，正在退出...");
                break;
            }

            let send_start = Instant::now();
            let filename = Self::get_filename(file_path);
            let subject = self.config.subject_template.as_ref().map_or_else(
                || format!("附件: {}", filename),
                |template| Self::process_template(template, &filename),
            );
            let text_content = self.config.text_template.as_ref().map_or_else(
                || format!("请查收附件: {}", filename),
                |template| Self::process_template(template, &filename),
            );
            let html_content = self
                .config
                .html_template
                .as_ref()
                .map(|template| Self::process_template(template, &filename));

            let empty_params = Parameters::default();
            if let Err(e) = client
                .mail_from(self.config.from.as_str(), &empty_params)
                .await
            {
                error!("设置发件人失败: {}", e);
                stats.increment_error("设置发件人失败", file_path);
                continue;
            }

            let recipients_str = self.config.to.as_str();
            let recipients: Vec<&str> = recipients_str
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();

            if recipients.is_empty() {
                error!(
                    "附件目录模式：没有有效的收件人地址 for {}: {}",
                    file_path, recipients_str
                );
                stats.increment_error("没有有效的收件人地址", file_path);
                continue;
            }

            let mut any_rcpt_succeeded = false;
            for recipient in &recipients {
                if let Err(e) = client.rcpt_to(recipient, &empty_params).await {
                    error!(
                        "附件目录模式：设置收件人 {} 失败 for {}: {}",
                        recipient, file_path, e
                    );
                    stats.increment_error(
                        &format!("设置收件人 {} 失败: {}", recipient, e),
                        file_path,
                    );
                } else {
                    info!(
                        "附件目录模式：设置收件人 {} 成功 for {}",
                        recipient, file_path
                    );
                    any_rcpt_succeeded = true;
                }
            }

            if !any_rcpt_succeeded {
                error!(
                    "附件目录模式：所有收件人均设置失败，跳过邮件发送 for {}",
                    file_path
                );
                // Errors already incremented per recipient
                continue;
            }

            let attachment_content = match fs::read(file_path) {
                Ok(content) => content,
                Err(e) => {
                    error!("读取附件文件失败: {}", e);
                    stats.increment_error("读取附件文件失败", file_path);
                    continue;
                }
            };

            let mut builder = MessageBuilder::new()
                .from(("", self.config.from.as_str()))
                .to(recipients) // Pass Vec<&str>
                .subject(&subject)
                .text_body(&text_content);
            if let Some(html) = &html_content {
                builder = builder.html_body(html);
            }
            let mime_type = infer::get_from_path(file_path)
                .ok()
                .flatten()
                .map_or("application/octet-stream", |k| k.mime_type());
            builder = builder.attachment(mime_type, &filename, &attachment_content[..]);

            let mail_content = match builder.write_to_vec() {
                Ok(content) => content,
                Err(e) => {
                    error!("生成邮件内容失败: {}", e);
                    stats.increment_error("生成邮件内容失败", file_path);
                    continue;
                }
            };

            match timeout(
                Duration::from_secs(self.config.smtp_timeout),
                client.data(&mail_content),
            )
            .await
            {
                Ok(Ok(_)) => {
                    info!("附件邮件发送成功！ 文件: {}", filename);
                    stats.email_count += 1;
                    stats.send_durations.push(send_start.elapsed());
                }
                Ok(Err(e)) => {
                    error!("邮件发送失败: {}, 文件: {}", e, file_path);
                    stats.increment_error(&format!("邮件发送失败: {}", e), file_path);
                }
                Err(_) => {
                    error!("邮件发送超时, 文件: {}", file_path);
                    stats.increment_error("邮件发送超时", file_path);
                }
            }

            if self.config.email_send_interval_ms > 0
                && (file_idx < files.len() - 1)
                && running.load(Ordering::SeqCst)
            {
                info!(
                    "附件目录模式：等待 {}ms 后发送下一个文件 (当前: {}/{})",
                    self.config.email_send_interval_ms,
                    file_idx + 1,
                    files.len()
                );
                let sleep_duration =
                    std::time::Duration::from_millis(self.config.email_send_interval_ms);
                let running_clone_for_sleep = running.clone();
                tokio::select! {
                    biased;
                    _ = async { loop { if !running_clone_for_sleep.load(Ordering::SeqCst) { break; } tokio::time::sleep(Duration::from_millis(100)).await; } } => {
                        warn!("附件目录模式：发送间隔休眠被中断 (文件 {}/{})", file_idx + 1, files.len());
                    }
                    _ = tokio::time::sleep(sleep_duration) => {}
                }
                if !running.load(Ordering::SeqCst) {
                    warn!(
                        "附件目录模式：收到中断信号，在间隔后退出 (文件 {}/{})",
                        file_idx + 1,
                        files.len()
                    );
                    break;
                }
            }
        }
        let _ = client.quit().await;
        stats.total_duration = start.elapsed();
        Ok(stats)
    }

    async fn execute_send_logic<T: AsyncRead + AsyncWrite + Unpin + Send>(
        &self,
        client: &mut SmtpClient<T>,
        attachment_path: &str,            // For logging and stats
        email_content: &EmailContent<'_>, // Email construction parameters
        stats: &mut Stats,                // To update stats
        running: Arc<AtomicBool>,         // To check for cancellation
    ) -> Result<()> {
        // Returns Result to indicate if overall send logic had issues, not individual email error
        if !running.load(Ordering::SeqCst) {
            warn!("execute_send_logic: 收到中断信号，正在退出...");
            return Ok(()); // Not an error, but operation stopped
        }

        let send_start = Instant::now();
        let empty_params = Parameters::default();

        if let Err(e) = client
            .mail_from(self.config.from.as_str(), &empty_params)
            .await
        {
            error!("设置发件人失败 for {}: {}", attachment_path, e);
            stats.increment_error(&format!("设置发件人失败: {}", e), attachment_path);
            return Ok(()); // Logged error, but function itself completed its attempt
        }

        let recipients: Vec<&str> = self
            .config
            .to
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        if recipients.is_empty() {
            error!(
                "没有有效的收件人地址 for {}: {}",
                attachment_path, self.config.to
            );
            stats.increment_error("没有有效的收件人地址", attachment_path);
            return Ok(());
        }

        let mut any_rcpt_succeeded = false;
        for recipient in &recipients {
            if let Err(e) = client.rcpt_to(recipient, &empty_params).await {
                error!(
                    "设置收件人 {} 失败 for {}: {}",
                    recipient, attachment_path, e
                );
                stats.increment_error(
                    &format!("设置收件人 {} 失败: {}", recipient, e),
                    attachment_path,
                );
            } else {
                info!("设置收件人 {} 成功 for {}", recipient, attachment_path);
                any_rcpt_succeeded = true;
            }
        }

        if !any_rcpt_succeeded {
            error!("所有收件人均设置失败，跳过邮件发送 for {}", attachment_path);
            // increment_error is already done per recipient
            return Ok(());
        }

        let attachment_content = match fs::read(attachment_path) {
            Ok(content) => content,
            Err(e) => {
                error!("读取附件文件失败 for {}: {}", attachment_path, e);
                stats.increment_error(&format!("读取附件文件失败: {}", e), attachment_path);
                return Ok(());
            }
        };

        let mut builder = MessageBuilder::new()
            .from(("", self.config.from.as_str()))
            .to(recipients) // Pass Vec<&str>
            .subject(email_content.subject)
            .text_body(email_content.text_content);

        if let Some(html) = email_content.html_content {
            builder = builder.html_body(html);
        }

        let mime_type = infer::get_from_path(attachment_path)
            .ok()
            .flatten()
            .map_or("application/octet-stream", |k| k.mime_type());
        builder = builder.attachment(mime_type, email_content.filename, &attachment_content[..]);

        let mail_content = match builder.write_to_vec() {
            Ok(content) => content,
            Err(e) => {
                error!("生成邮件内容失败 for {}: {}", attachment_path, e);
                stats.increment_error(&format!("生成邮件内容失败: {}", e), attachment_path);
                return Ok(());
            }
        };

        match timeout(
            Duration::from_secs(self.config.smtp_timeout),
            client.data(&mail_content),
        )
        .await
        {
            Ok(Ok(_)) => {
                info!("附件邮件发送成功！Path: {}", attachment_path);
                stats.email_count += 1;
                stats.send_durations.push(send_start.elapsed());
            }
            Ok(Err(e)) => {
                error!("邮件发送失败 for {}: {}", attachment_path, e);
                stats.increment_error(&format!("邮件发送失败: {}", e), attachment_path);
            }
            Err(_) => {
                error!("邮件发送超时 for {}", attachment_path);
                stats.increment_error("邮件发送超时", attachment_path);
            }
        }
        // client.quit() is handled by the caller of execute_send_logic
        Ok(())
    }

    async fn send_attachment_with_cancel(
        &self,
        attachment_path: &str,
        running: Arc<AtomicBool>,
    ) -> Result<Stats> {
        info!("准备发送附件：{}", attachment_path);
        let mut stats = Stats::new();
        let start = Instant::now();

        if !Path::new(attachment_path).exists() {
            error!("附件文件不存在: {}", attachment_path);
            stats.increment_error("附件文件不存在", attachment_path); // Record error in stats
            return Ok(stats); // Return stats with error instead of Err(anyhow!)
        }

        let filename = Self::get_filename(attachment_path);
        let subject = self.config.subject_template.as_ref().map_or_else(
            || format!("附件: {}", filename),
            |template| Self::process_template(template, &filename),
        );
        let text_content = self.config.text_template.as_ref().map_or_else(
            || format!("请查收附件: {}", filename),
            |template| Self::process_template(template, &filename),
        );
        let html_content = self
            .config
            .html_template
            .as_ref()
            .map(|template| Self::process_template(template, &filename));

        info!(
            "连接SMTP服务器: {}:{}",
            self.config.smtp_server, self.config.port
        );
        let use_tls = self.config.use_tls || self.config.port == 465;

        // No longer need client_result to be a single variable for different types.
        // We will handle connection and then call execute_send_logic within each branch.

        if self.config.auth_mode {
            if let (Some(username), Some(password)) = (&self.config.username, &self.config.password)
            {
                info!("使用账号登录模式: {}", username);
                if use_tls {
                    info!("使用TLS连接 (认证模式)");
                    let mut client_builder =
                        SmtpClientBuilder::new(self.config.smtp_server.as_str(), self.config.port)
                            .credentials((username.as_str(), password.as_str()));
                    client_builder = if self.config.port == 465 {
                        client_builder.implicit_tls(true)
                    } else {
                        client_builder.implicit_tls(false) // For STARTTLS
                    };
                    if self.config.accept_invalid_certs {
                        client_builder = client_builder.allow_invalid_certs();
                    }
                    match timeout(
                        Duration::from_secs(self.config.smtp_timeout),
                        client_builder.connect(),
                    )
                    .await
                    {
                        Ok(Ok(mut client)) => {
                            // client is SmtpClient<TlsStream<TcpStream>>
                            let email_content = EmailContent {
                                filename: &filename,
                                subject: &subject,
                                text_content: &text_content,
                                html_content: &html_content,
                            };
                            let _ = self
                                .execute_send_logic(
                                    &mut client,
                                    attachment_path,
                                    &email_content,
                                    &mut stats,
                                    running.clone(),
                                )
                                .await;
                            let _ = client.quit().await;
                        }
                        Ok(Err(e)) => {
                            error!("SMTP认证连接失败: {}", e);
                            stats.increment_error(
                                &format!("SMTP认证连接失败: {}", e),
                                attachment_path,
                            );
                        }
                        Err(_) => {
                            error!("SMTP连接超时 (认证模式)");
                            stats.increment_error("SMTP连接超时 (认证模式)", attachment_path);
                        }
                    }
                } else {
                    error!("不支持使用非TLS连接进行账号登录，请设置--use-tls参数或使用465端口");
                    stats.increment_error("认证失败: 需要TLS连接", attachment_path);
                }
            } else {
                error!("账号登录模式启用但缺少用户名或密码");
                stats.increment_error("认证失败: 缺少用户名或密码", attachment_path);
            }
        } else {
            // Non-authenticated mode
            let mut client_builder =
                SmtpClientBuilder::new(self.config.smtp_server.as_str(), self.config.port);
            if use_tls {
                info!("使用TLS连接 (非认证模式)");
                client_builder = if self.config.port == 465 {
                    client_builder.implicit_tls(true)
                } else {
                    client_builder.implicit_tls(false) // For STARTTLS
                };
                if self.config.accept_invalid_certs {
                    client_builder = client_builder.allow_invalid_certs();
                }
                match timeout(
                    Duration::from_secs(self.config.smtp_timeout),
                    client_builder.connect(),
                )
                .await
                {
                    Ok(Ok(mut client)) => {
                        // client is SmtpClient<TlsStream<TcpStream>>
                        let email_content = EmailContent {
                            filename: &filename,
                            subject: &subject,
                            text_content: &text_content,
                            html_content: &html_content,
                        };
                        let _ = self
                            .execute_send_logic(
                                &mut client,
                                attachment_path,
                                &email_content,
                                &mut stats,
                                running.clone(),
                            )
                            .await;
                        let _ = client.quit().await;
                    }
                    Ok(Err(e)) => {
                        error!("SMTP非认证TLS连接失败: {}", e);
                        stats.increment_error(
                            &format!("SMTP非认证TLS连接失败: {}", e),
                            attachment_path,
                        );
                    }
                    Err(_) => {
                        error!("SMTP连接超时 (非认证TLS)");
                        stats.increment_error("SMTP连接超时 (非认证TLS)", attachment_path);
                    }
                }
            } else {
                // Plain connection
                info!("使用Plain连接 (非认证模式)");
                match timeout(
                    Duration::from_secs(self.config.smtp_timeout),
                    client_builder.connect_plain(),
                )
                .await
                {
                    Ok(Ok(mut client)) => {
                        // client is SmtpClient<TcpStream>
                        let email_content = EmailContent {
                            filename: &filename,
                            subject: &subject,
                            text_content: &text_content,
                            html_content: &html_content,
                        };
                        let _ = self
                            .execute_send_logic(
                                &mut client,
                                attachment_path,
                                &email_content,
                                &mut stats,
                                running.clone(),
                            )
                            .await;
                        let _ = client.quit().await;
                    }
                    Ok(Err(e)) => {
                        error!("SMTP非认证Plain连接失败: {}", e);
                        stats.increment_error(
                            &format!("SMTP非认证Plain连接失败: {}", e),
                            attachment_path,
                        );
                    }
                    Err(_) => {
                        error!("SMTP连接超时 (非认证Plain)");
                        stats.increment_error("SMTP连接超时 (非认证Plain)", attachment_path);
                    }
                }
            }
        }

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
        let chunk_size = files.len().div_ceil(num_processes);

        let mut handles = vec![];
        for (i, chunk) in files.chunks(chunk_size).enumerate() {
            let chunk = chunk.to_vec();
            let config = self.config.clone();
            let running = running.clone();

            let handle = task::spawn(async move {
                let mut group_stats: GroupStats = (0, Vec::new(), Vec::new(), Vec::new());
                let mut current_batch = Vec::new(); // Correctly declared here

                // For non-auth mode with connection reuse (client_opt)
                // We will stick to SmtpClient<tokio::net::TcpStream> for client_opt.
                // If TLS is needed in non-auth mode, we won't reuse client_opt; new connection per batch.
                let mut client_opt: Option<SmtpClient<tokio::net::TcpStream>> = None;

                let use_tls = config.use_tls || config.port == 465;

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
                            chunk.len().div_ceil(config.batch_size),
                            current_batch.len()
                        );

                        if config.auth_mode {
                            client_opt = None; // Ensure no reuse from a previous non-auth iteration
                            if let (Some(username), Some(password)) =
                                (&config.username, &config.password)
                            {
                                if use_tls {
                                    let mut client_builder = SmtpClientBuilder::new(
                                        config.smtp_server.as_str(),
                                        config.port,
                                    )
                                    .credentials((username.as_str(), password.as_str()));
                                    client_builder = if config.port == 465 {
                                        client_builder.implicit_tls(true)
                                    } else {
                                        client_builder.implicit_tls(false)
                                    };
                                    if config.accept_invalid_certs {
                                        client_builder = client_builder.allow_invalid_certs();
                                    }

                                    match timeout(
                                        Duration::from_secs(config.smtp_timeout),
                                        client_builder.connect(),
                                    )
                                    .await
                                    {
                                        Ok(Ok(mut client)) => {
                                            // client is SmtpClient<TlsStream<TcpStream>>
                                            if let Err(e) = Self::process_batch_with_tls_client(
                                                &config,
                                                &current_batch,
                                                &mut client,
                                                &mut group_stats,
                                                i + 1,
                                                running.clone(),
                                            )
                                            .await
                                            {
                                                error!("进程组 {}: TLS批量发送失败: {}", i + 1, e);
                                                for file_path_in_batch in &current_batch {
                                                    group_stats.3.push((
                                                        format!("TLS批量处理错误: {}", e),
                                                        file_path_in_batch.clone(),
                                                    ));
                                                }
                                            }
                                            let _ = client.quit().await;
                                        }
                                        Ok(Err(e)) => {
                                            error!("进程组 {}: SMTP认证连接失败: {}", i + 1, e);
                                            for file_path_in_batch in &current_batch {
                                                group_stats.3.push((
                                                    "SMTP认证连接失败".to_string(),
                                                    file_path_in_batch.clone(),
                                                ));
                                            }
                                        }
                                        Err(_) => {
                                            error!("进程组 {}: SMTP认证连接超时", i + 1);
                                            for file_path_in_batch in &current_batch {
                                                group_stats.3.push((
                                                    "SMTP认证连接超时".to_string(),
                                                    file_path_in_batch.clone(),
                                                ));
                                            }
                                        }
                                    }
                                } else {
                                    error!("进程组 {}: 认证模式不支持非TLS连接.", i + 1);
                                    for file_path_in_batch in &current_batch {
                                        group_stats.3.push((
                                            "认证失败: 需要TLS".to_string(),
                                            file_path_in_batch.clone(),
                                        ));
                                    }
                                }
                            } else {
                                error!("进程组 {}: 认证模式缺少用户名或密码.", i + 1);
                                for file_path_in_batch in &current_batch {
                                    group_stats.3.push((
                                        "认证失败: 凭证不完整".to_string(),
                                        file_path_in_batch.clone(),
                                    ));
                                }
                            }
                        } else {
                            // Non-authenticated mode
                            if use_tls {
                                // Non-auth + TLS: no client_opt reuse, new connection per batch
                                client_opt = None;
                                info!("进程组 {}: 非认证模式，使用TLS连接 (无持久化)", i + 1);
                                let mut client_builder = SmtpClientBuilder::new(
                                    config.smtp_server.as_str(),
                                    config.port,
                                );
                                client_builder = if config.port == 465 {
                                    client_builder.implicit_tls(true)
                                } else {
                                    client_builder.implicit_tls(false)
                                };
                                if config.accept_invalid_certs {
                                    client_builder = client_builder.allow_invalid_certs();
                                }

                                match timeout(
                                    Duration::from_secs(config.smtp_timeout),
                                    client_builder.connect(),
                                )
                                .await
                                {
                                    Ok(Ok(mut client)) => {
                                        // client is SmtpClient<TlsStream<TcpStream>>
                                        // process_batch_with_tls_client is generic enough for SmtpClient<TlsStream<TcpStream>>
                                        if let Err(e) = Self::process_batch_with_tls_client(
                                            &config,
                                            &current_batch,
                                            &mut client,
                                            &mut group_stats,
                                            i + 1,
                                            running.clone(),
                                        )
                                        .await
                                        {
                                            error!(
                                                "进程组 {}: 非认证TLS批量发送失败: {}",
                                                i + 1,
                                                e
                                            );
                                            for file_path_in_batch in &current_batch {
                                                group_stats.3.push((
                                                    format!("非认证TLS批量处理错误: {}", e),
                                                    file_path_in_batch.clone(),
                                                ));
                                            }
                                        }
                                        let _ = client.quit().await;
                                    }
                                    Ok(Err(e)) => {
                                        error!("进程组 {}: SMTP非认证TLS连接失败: {}", i + 1, e);
                                        for file_path_in_batch in &current_batch {
                                            group_stats.3.push((
                                                "SMTP非认证TLS连接失败".to_string(),
                                                file_path_in_batch.clone(),
                                            ));
                                        }
                                    }
                                    Err(_) => {
                                        error!("进程组 {}: SMTP非认证TLS连接超时", i + 1);
                                        for file_path_in_batch in &current_batch {
                                            group_stats.3.push((
                                                "SMTP非认证TLS连接超时".to_string(),
                                                file_path_in_batch.clone(),
                                            ));
                                        }
                                    }
                                }
                            } else {
                                // Non-auth + Plain: use client_opt for potential reuse
                                if client_opt.is_none() {
                                    info!(
                                        "进程组 {}: 连接SMTP服务器: {}:{} (非认证模式, Plain)",
                                        i + 1,
                                        config.smtp_server,
                                        config.port
                                    );
                                    let client_builder = SmtpClientBuilder::new(
                                        config.smtp_server.as_str(),
                                        config.port,
                                    );
                                    match timeout(
                                        Duration::from_secs(config.smtp_timeout),
                                        client_builder.connect_plain(),
                                    )
                                    .await
                                    {
                                        Ok(Ok(client)) => client_opt = Some(client),
                                        Ok(Err(e)) => {
                                            error!(
                                                "进程组 {}: SMTP连接失败 (非认证Plain): {}",
                                                i + 1,
                                                e
                                            );
                                            for file_path_in_batch in &current_batch {
                                                group_stats.3.push((
                                                    "SMTP连接失败Plain".to_string(),
                                                    file_path_in_batch.clone(),
                                                ));
                                            }
                                        }
                                        Err(_) => {
                                            error!("进程组 {}: SMTP连接超时 (非认证Plain).", i + 1);
                                            for file_path_in_batch in &current_batch {
                                                group_stats.3.push((
                                                    "SMTP连接超时Plain".to_string(),
                                                    file_path_in_batch.clone(),
                                                ));
                                            }
                                        }
                                    }
                                }

                                if let Some(ref mut client) = client_opt {
                                    // client is SmtpClient<TcpStream>
                                    let (successes, failures, should_reset_connection) =
                                        Self::send_batch_emails(
                                            &config,
                                            &current_batch,
                                            client,
                                            running.clone(),
                                        )
                                        .await;

                                    group_stats.0 += successes.len();
                                    group_stats.1.extend(successes.iter().map(|(pd, _)| *pd));
                                    group_stats.2.extend(successes.iter().map(|(_, sd)| *sd));
                                    for (error_message, file_path_string) in failures {
                                        group_stats.3.push((error_message, file_path_string));
                                    }

                                    // 使用函数返回的连接状态标志，立即响应SMTP协议要求
                                    if should_reset_connection {
                                        warn!(
                                            "进程组 {}: 检测到需要重置连接的SMTP错误（如421），立即重置连接",
                                            i + 1
                                        );
                                        // 立即重置连接，下个批次将重新建立
                                        client_opt = None;
                                    }

                                    // batch-size=1时强制关闭连接，避免连接重用
                                    if config.batch_size == 1 {
                                        info!(
                                            "进程组 {}: batch-size=1，强制关闭连接以确保下一批次建立新连接",
                                            i + 1
                                        );
                                        client_opt = None;
                                    }
                                } else {
                                    info!(
                                        "进程组 {} (非认证Plain): SMTP连接不可用，跳过当前批次发送",
                                        i + 1
                                    );
                                }
                            }
                        }
                        current_batch.clear();
                        if config.email_send_interval_ms > 0
                            && j < chunk.len() - 1
                            && running.load(Ordering::SeqCst)
                        {
                            info!(
                                "进程组 {}: 批处理尝试完毕。等待 {}ms (当前文件索引 {}/{})",
                                i + 1,
                                config.email_send_interval_ms,
                                j + 1,
                                chunk.len()
                            );
                            let sleep_duration =
                                std::time::Duration::from_millis(config.email_send_interval_ms);
                            let running_clone_for_sleep = running.clone();
                            tokio::select! {
                                biased;
                                _ = async { loop { if !running_clone_for_sleep.load(Ordering::SeqCst) { break; } tokio::time::sleep(Duration::from_millis(100)).await; } } => { warn!("进程组 {}: 任务间隔休眠被中断 (文件 {}/{})", i + 1, j + 1, chunk.len()); }
                                _ = tokio::time::sleep(sleep_duration) => {}
                            }
                            if !running.load(Ordering::SeqCst) {
                                warn!(
                                    "进程组 {}: 收到中断信号，在任务间隔后退出 (文件 {}/{})",
                                    i + 1,
                                    j + 1,
                                    chunk.len()
                                );
                                break;
                            }
                        }
                    }
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
        let dir = match &self.config.dir {
            Some(dir_path) => dir_path,
            None => {
                info!("使用附件模式，跳过邮件文件扫描");
                return Ok(files);
            }
        };
        info!("开始扫描目录: {}", dir);
        for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
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
        running: Arc<AtomicBool>,
    ) -> (Vec<(Duration, Duration)>, Vec<(String, String)>, bool) {
        let mut successes = Vec::new();
        let mut failures = Vec::new();
        let mut connection_should_reset = false; // 跟踪连接是否需要重置
        let mut anonymizer = if config.anonymize_emails {
            Some(EmailAnonymizer::new(&config.anonymize_domain))
        } else {
            None
        };

        let base_recipients_str = config.to.as_str();
        let base_recipients: Vec<&str> = base_recipients_str
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        if base_recipients.is_empty() {
            warn!(
                "全局收件人地址 (config.to) 为空或无效: {}",
                base_recipients_str
            );
            // If global 'to' is empty, all emails in this batch will fail for missing recipients.
            // We'll add an error for each file to reflect this in the loop if needed.
        }

        for (email_idx, file_path) in files.iter().enumerate() {
            if !running.load(Ordering::SeqCst) {
                warn!("send_batch_emails: 收到中断信号，正在退出批处理...");
                break;
            }
            let mut had_error_this_email = false;
            let mut current_file_parse_duration: Option<Duration> = None;
            let parse_start = Instant::now();

            // current_recipients will be the same as base_recipients for this function's scope
            // as it's not modified per email file here.
            let current_recipients = base_recipients.clone();

            let content_read_result = fs::read(file_path);

            let content = match content_read_result {
                Ok(c) => {
                    current_file_parse_duration = Some(parse_start.elapsed());
                    if let Some(anonymizer_ref) = anonymizer.as_mut() {
                        info!("对邮件内容进行邮箱匿名化处理: {}", file_path);
                        anonymizer_ref.anonymize_binary(&c)
                    } else {
                        c
                    }
                }
                Err(e) => {
                    error!("读取文件 {} 失败: {}", file_path, e);
                    failures.push((format!("读取文件失败: {}", e), file_path.to_string()));
                    Self::save_failed_email(config, file_path);
                    had_error_this_email = true;
                    Vec::new() // dummy content
                }
            };

            if !had_error_this_email {
                let parse_duration_final =
                    current_file_parse_duration.unwrap_or_else(|| parse_start.elapsed());
                let message = match MessageParser::default().parse(&content) {
                    Some(msg) => msg,
                    None => {
                        error!("无法解析邮件文件: {}", file_path);
                        failures.push(("无法解析邮件文件".to_string(), file_path.to_string()));
                        Self::save_failed_email(config, file_path);
                        had_error_this_email = true;
                        MessageParser::default().parse(b"Subject: error").unwrap()
                        // dummy message
                    }
                };

                if !had_error_this_email {
                    let send_start = Instant::now();
                    let empty_params = Parameters::default();
                    let mut email_send_op_failed = false;

                    if current_recipients.is_empty() {
                        // This check is important if base_recipients could be empty
                        error!(
                            "send_batch_emails: 没有有效的收件人地址 for {}: {}",
                            file_path, config.to
                        );
                        failures.push((
                            format!("没有有效的收件人地址: {}", config.to),
                            file_path.to_string(),
                        ));
                        Self::save_failed_email(config, file_path);
                        email_send_op_failed = true;
                    }

                    if !email_send_op_failed {
                        if let Err(e) = client.mail_from(config.from.as_str(), &empty_params).await
                        {
                            error!("send_batch_emails: 设置发件人失败 for {}: {}", file_path, e);
                            let error_msg = format!("设置发件人失败: {}", e);
                            failures.push((error_msg.clone(), file_path.to_string()));
                            Self::save_failed_email(config, file_path);
                            email_send_op_failed = true;

                            // 检测关键SMTP错误，这些错误表示服务器要求断开连接
                            if error_msg.contains("421") ||  // Service not available, closing transmission channel
                               error_msg.contains("Cannot accept further commands") ||
                               error_msg.contains("Broken pipe") ||
                               error_msg.contains("Connection reset") ||
                               error_msg.contains("Unparseable SMTP reply")
                            {
                                // 连接已损坏的信号
                                warn!(
                                    "send_batch_emails: 检测到需要重置连接的SMTP错误: {}",
                                    error_msg
                                );
                                connection_should_reset = true;
                                // 立即退出当前批次，避免更多无效尝试
                                break;
                            }
                        }
                    }

                    if !email_send_op_failed {
                        let mut any_rcpt_succeeded = false;
                        for recipient in &current_recipients {
                            if let Err(e) = client.rcpt_to(recipient, &empty_params).await {
                                error!(
                                    "send_batch_emails: 设置收件人 {} 失败 for {}: {}",
                                    recipient, file_path, e
                                );
                                failures.push((
                                    format!("设置收件人 {} 失败: {}", recipient, e),
                                    file_path.to_string(),
                                ));
                            } else {
                                info!(
                                    "send_batch_emails: 设置收件人 {} 成功 for {}",
                                    recipient, file_path
                                );
                                any_rcpt_succeeded = true;
                            }
                        }
                        if !any_rcpt_succeeded && !current_recipients.is_empty() {
                            error!(
                                "send_batch_emails: 所有收件人均设置失败，跳过邮件发送 for {}",
                                file_path
                            );
                            Self::save_failed_email(config, file_path);
                            email_send_op_failed = true;
                        }
                    }

                    if !email_send_op_failed {
                        let mail_data_to_send = if config.keep_headers {
                            info!("使用原始邮件头发送邮件: {}", file_path);
                            content.clone()
                        } else if config.modify_headers {
                            info!("修改邮件头并发送邮件: {}", file_path);
                            let subject = message.subject().unwrap_or("No Subject").to_string();
                            let text_content = message.body_text(0).unwrap_or_default().to_string();
                            let html_content = message.body_html(0).map(|s| s.to_string());
                            let mut builder = MessageBuilder::new()
                                .from(("", config.from.as_str()))
                                .to(current_recipients.clone())
                                .subject(&subject)
                                .text_body(&text_content);
                            if let Some(html) = &html_content {
                                builder = builder.html_body(html);
                            }
                            match builder.write_to_vec() {
                                Ok(m_content) => m_content,
                                Err(e) => {
                                    error!("构建邮件内容失败 for {}: {}", file_path, e);
                                    failures.push((
                                        format!("构建邮件内容失败: {}", e),
                                        file_path.to_string(),
                                    ));
                                    Self::save_failed_email(config, file_path);
                                    email_send_op_failed = true;
                                    Vec::new()
                                }
                            }
                        } else {
                            // 修复附件丢失问题：在默认模式下也使用原始邮件内容来保持附件和完整的MIME结构
                            info!("使用原始邮件内容发送（保持附件和MIME结构）: {}", file_path);
                            content.clone()
                        };

                        if !email_send_op_failed {
                            match timeout(
                                Duration::from_secs(config.smtp_timeout),
                                client.data(&mail_data_to_send),
                            )
                            .await
                            {
                                Ok(Ok(_)) => {
                                    info!("邮件发送成功！: {}", file_path);
                                    successes.push((parse_duration_final, send_start.elapsed()));
                                }
                                Ok(Err(e)) => {
                                    error!("邮件发送失败 for file {}: {}", file_path, e);
                                    let error_msg = format!("邮件发送失败: {}", e);
                                    failures.push((error_msg.clone(), file_path.to_string()));
                                    Self::save_failed_email(config, file_path);

                                    // 检测关键SMTP错误
                                    if error_msg.contains("421")
                                        || error_msg.contains("Cannot accept further commands")
                                        || error_msg.contains("Broken pipe")
                                        || error_msg.contains("Connection reset")
                                        || error_msg.contains("Unparseable SMTP reply")
                                    {
                                        warn!(
                                            "send_batch_emails: 数据发送时检测到连接问题: {}",
                                            error_msg
                                        );
                                        connection_should_reset = true;
                                        break;
                                    }
                                }
                                Err(_) => {
                                    error!("邮件发送超时 for file: {}", file_path);
                                    failures
                                        .push(("邮件发送超时".to_string(), file_path.to_string()));
                                    Self::save_failed_email(config, file_path);
                                }
                            }
                        }
                    }
                }
            }

            // 添加RSET命令：如果还有更多邮件要发送，重置SMTP状态
            if email_idx < files.len() - 1
                && running.load(Ordering::SeqCst)
                && !connection_should_reset
            {
                info!(
                    "send_batch_emails: 发送RSET命令重置SMTP状态 (批次邮件 {}/{})",
                    email_idx + 1,
                    files.len()
                );
                if let Err(e) = client.rset().await {
                    warn!(
                        "send_batch_emails: RSET命令发送失败 (批次邮件 {}/{}): {}",
                        email_idx + 1,
                        files.len(),
                        e
                    );
                    // RSET失败通常意味着连接有问题，标记需要重置连接
                    connection_should_reset = true;
                    break;
                }
            }

            if config.email_send_interval_ms > 0
                && email_idx < files.len() - 1
                && running.load(Ordering::SeqCst)
            {
                info!(
                    "send_batch_emails: 等待 {}ms 后发送下一封邮件 (当前批次中邮件索引: {}/{})",
                    config.email_send_interval_ms,
                    email_idx + 1,
                    files.len()
                );
                let sleep_duration =
                    std::time::Duration::from_millis(config.email_send_interval_ms);
                let running_clone_for_sleep = running.clone();
                tokio::select! {
                    biased;
                    _ = async { loop { if !running_clone_for_sleep.load(Ordering::SeqCst) { break; } tokio::time::sleep(Duration::from_millis(100)).await; } } => {
                        warn!("send_batch_emails: 邮件发送间隔休眠被中断 (批次邮件 {}/{})", email_idx + 1, files.len());
                    }
                    _ = tokio::time::sleep(sleep_duration) => {}
                }
                if !running.load(Ordering::SeqCst) {
                    warn!(
                        "send_batch_emails: 收到中断信号，在邮件间隔后退出批处理 (批次邮件 {}/{})",
                        email_idx + 1,
                        files.len()
                    );
                    break;
                }
            }
        }
        (successes, failures, connection_should_reset)
    }

    async fn process_batch_with_tls_client<S: AsyncRead + AsyncWrite + Unpin + Send>(
        config: &Config,
        files: &[String],
        client: &mut SmtpClient<S>,
        group_stats: &mut GroupStats,
        process_group_id: usize,
        running: Arc<AtomicBool>,
    ) -> Result<()> {
        let mut anonymizer = if config.anonymize_emails {
            Some(EmailAnonymizer::new(&config.anonymize_domain))
        } else {
            None
        };

        let base_recipients_str = config.to.as_str();
        let base_recipients: Vec<&str> = base_recipients_str
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        if base_recipients.is_empty() {
            warn!(
                "进程组 {}: 全局收件人地址 (config.to) 为空或无效: {}",
                process_group_id, base_recipients_str
            );
            // If global 'to' is empty, all emails in this batch will fail for missing recipients.
            // We'll add an error for each file to reflect this in the loop if needed.
        }

        for (email_idx, file_path) in files.iter().enumerate() {
            if !running.load(Ordering::SeqCst) {
                warn!(
                    "进程组 {}: process_batch_with_tls_client: 收到中断信号，正在退出批处理...",
                    process_group_id
                );
                break;
            }
            let mut had_error_this_email = false;
            let mut current_file_parse_duration: Option<Duration> = None;
            let parse_start = Instant::now();

            let current_recipients = base_recipients.clone();

            let content_read_result = fs::read(file_path);

            let content = match content_read_result {
                Ok(c) => {
                    current_file_parse_duration = Some(parse_start.elapsed());
                    if let Some(anonymizer_ref) = anonymizer.as_mut() {
                        info!(
                            "进程组 {}: 对邮件内容进行邮箱匿名化处理: {}",
                            process_group_id, file_path
                        );
                        anonymizer_ref.anonymize_binary(&c)
                    } else {
                        c
                    }
                }
                Err(e) => {
                    error!(
                        "进程组 {}: 读取文件 {} 失败: {}",
                        process_group_id, file_path, e
                    );
                    group_stats
                        .3
                        .push((format!("读取文件失败: {}", e), file_path.to_string()));
                    Self::save_failed_email(config, file_path);
                    had_error_this_email = true;
                    Vec::new()
                }
            };

            if !had_error_this_email {
                let parse_duration_final =
                    current_file_parse_duration.unwrap_or_else(|| parse_start.elapsed());
                let message = match MessageParser::default().parse(&content) {
                    Some(msg) => msg,
                    None => {
                        error!(
                            "进程组 {}: 无法解析邮件文件: {}",
                            process_group_id, file_path
                        );
                        group_stats
                            .3
                            .push(("无法解析邮件文件".to_string(), file_path.to_string()));
                        Self::save_failed_email(config, file_path);
                        had_error_this_email = true;
                        MessageParser::default().parse(b"Subject: error").unwrap()
                    }
                };

                if !had_error_this_email {
                    let send_start = Instant::now();
                    let empty_params = Parameters::default();
                    let mut email_send_op_failed = false;

                    if current_recipients.is_empty() {
                        error!(
                            "进程组 {}: 没有有效的收件人地址 for {}: {}",
                            process_group_id, file_path, config.to
                        );
                        group_stats.3.push((
                            format!("没有有效的收件人地址: {}", config.to),
                            file_path.to_string(),
                        ));
                        Self::save_failed_email(config, file_path);
                        email_send_op_failed = true;
                    }

                    if !email_send_op_failed {
                        if let Err(e) = client.mail_from(config.from.as_str(), &empty_params).await
                        {
                            error!(
                                "进程组 {}: 设置发件人失败 for {}: {}",
                                process_group_id, file_path, e
                            );
                            let error_msg = format!("设置发件人失败: {}", e);
                            group_stats
                                .3
                                .push((error_msg.clone(), file_path.to_string()));
                            Self::save_failed_email(config, file_path);
                            email_send_op_failed = true;

                            // 检测关键SMTP错误，特别是421等要求断开连接的错误
                            if error_msg.contains("421")
                                || error_msg.contains("Cannot accept further commands")
                                || error_msg.contains("Broken pipe")
                                || error_msg.contains("Connection reset")
                                || error_msg.contains("Unparseable SMTP reply")
                                || error_msg.contains("timeout")
                                || error_msg.contains("超时")
                            {
                                warn!(
                                    "进程组 {}: 设置发件人时检测到需要断开连接的SMTP错误，提前退出批次: {}",
                                    process_group_id, error_msg
                                );
                                break;
                            }
                        }
                    }

                    if !email_send_op_failed {
                        let mut any_rcpt_succeeded = false;
                        for recipient in &current_recipients {
                            if let Err(e) = client.rcpt_to(recipient, &empty_params).await {
                                error!(
                                    "进程组 {}: 设置收件人 {} 失败 for {}: {}",
                                    process_group_id, recipient, file_path, e
                                );
                                group_stats.3.push((
                                    format!("设置收件人 {} 失败: {}", recipient, e),
                                    file_path.to_string(),
                                ));
                            } else {
                                info!(
                                    "进程组 {}: 设置收件人 {} 成功 for {}",
                                    process_group_id, recipient, file_path
                                );
                                any_rcpt_succeeded = true;
                            }
                        }
                        if !any_rcpt_succeeded && !current_recipients.is_empty() {
                            error!(
                                "进程组 {}: 所有收件人均设置失败，跳过邮件发送 for {}",
                                process_group_id, file_path
                            );
                            Self::save_failed_email(config, file_path);
                            email_send_op_failed = true;
                        }
                    }

                    if !email_send_op_failed {
                        let mail_data_to_send = if config.keep_headers {
                            info!(
                                "进程组 {}: 使用原始邮件头发送邮件: {}",
                                process_group_id, file_path
                            );
                            content.clone()
                        } else if config.modify_headers {
                            info!(
                                "进程组 {}: 修改邮件头并发送邮件: {}",
                                process_group_id, file_path
                            );
                            let subject = message.subject().unwrap_or("No Subject").to_string();
                            let text_content = message.body_text(0).unwrap_or_default().to_string();
                            let html_content = message.body_html(0).map(|s| s.to_string());
                            let mut builder = MessageBuilder::new()
                                .from(("", config.from.as_str()))
                                .to(current_recipients.clone())
                                .subject(&subject)
                                .text_body(&text_content);
                            if let Some(html) = &html_content {
                                builder = builder.html_body(html);
                            }
                            match builder.write_to_vec() {
                                Ok(m_content) => m_content,
                                Err(e) => {
                                    error!(
                                        "进程组 {}: 构建邮件内容失败 for {}: {}",
                                        process_group_id, file_path, e
                                    );
                                    group_stats.3.push((
                                        format!("构建邮件内容失败: {}", e),
                                        file_path.to_string(),
                                    ));
                                    Self::save_failed_email(config, file_path);
                                    email_send_op_failed = true;
                                    Vec::new()
                                }
                            }
                        } else {
                            // 修复附件丢失问题：在默认模式下也使用原始邮件内容来保持附件和完整的MIME结构
                            info!(
                                "进程组 {}: 使用原始邮件内容发送（保持附件和MIME结构）: {}",
                                process_group_id, file_path
                            );
                            content.clone()
                        };

                        if !email_send_op_failed {
                            match timeout(
                                Duration::from_secs(config.smtp_timeout),
                                client.data(&mail_data_to_send),
                            )
                            .await
                            {
                                Ok(Ok(_)) => {
                                    info!(
                                        "进程组 {}: 邮件发送成功！: {}",
                                        process_group_id, file_path
                                    );
                                    group_stats.0 += 1;
                                    group_stats.1.push(parse_duration_final);
                                    group_stats.2.push(send_start.elapsed());
                                }
                                Ok(Err(e)) => {
                                    error!(
                                        "进程组 {}: 邮件发送失败 for file {}: {}",
                                        process_group_id, file_path, e
                                    );
                                    let error_msg = format!("邮件发送失败: {}", e);
                                    group_stats
                                        .3
                                        .push((error_msg.clone(), file_path.to_string()));
                                    Self::save_failed_email(config, file_path);

                                    // 检测关键SMTP错误，特别是421等要求断开连接的错误
                                    if error_msg.contains("421")
                                        || error_msg.contains("Cannot accept further commands")
                                        || error_msg.contains("Broken pipe")
                                        || error_msg.contains("Connection reset")
                                        || error_msg.contains("Unparseable SMTP reply")
                                        || error_msg.contains("timeout")
                                        || error_msg.contains("超时")
                                    {
                                        warn!(
                                            "进程组 {}: 检测到需要断开连接的SMTP错误，提前退出批次: {}",
                                            process_group_id, error_msg
                                        );
                                        break;
                                    }
                                }
                                Err(_) => {
                                    error!(
                                        "进程组 {}: 邮件发送超时 for file: {}",
                                        process_group_id, file_path
                                    );
                                    group_stats
                                        .3
                                        .push(("邮件发送超时".to_string(), file_path.to_string()));
                                    Self::save_failed_email(config, file_path);
                                }
                            }
                        }
                    }
                }
            }

            // 添加RSET命令：如果还有更多邮件要发送，重置SMTP状态
            if email_idx < files.len() - 1 && running.load(Ordering::SeqCst) {
                info!(
                    "进程组 {}: 发送RSET命令重置SMTP状态 (批次邮件 {}/{})",
                    process_group_id,
                    email_idx + 1,
                    files.len()
                );
                if let Err(e) = client.rset().await {
                    warn!(
                        "进程组 {}: RSET命令发送失败 (批次邮件 {}/{}): {}",
                        process_group_id,
                        email_idx + 1,
                        files.len(),
                        e
                    );
                    // RSET失败通常意味着连接有问题，提前退出批次
                    break;
                }
            }

            if config.email_send_interval_ms > 0
                && email_idx < files.len() - 1
                && running.load(Ordering::SeqCst)
            {
                info!(
                    "进程组 {}: 等待 {}ms 后发送下一封邮件 (当前批次中邮件索引: {}/{})",
                    process_group_id,
                    config.email_send_interval_ms,
                    email_idx + 1,
                    files.len()
                );
                let sleep_duration =
                    std::time::Duration::from_millis(config.email_send_interval_ms);
                let running_clone_for_sleep = running.clone();
                tokio::select! {
                    biased;
                    _ = async { loop { if !running_clone_for_sleep.load(Ordering::SeqCst) { break; } tokio::time::sleep(Duration::from_millis(100)).await; } } => {
                        warn!("进程组 {}: 邮件发送间隔休眠被中断 (批次邮件 {}/{})", process_group_id, email_idx + 1, files.len());
                    }
                    _ = tokio::time::sleep(sleep_duration) => {}
                }
                if !running.load(Ordering::SeqCst) {
                    warn!(
                        "进程组 {}: 收到中断信号，在邮件间隔后退出批处理 (批次邮件 {}/{})",
                        process_group_id,
                        email_idx + 1,
                        files.len()
                    );
                    break;
                }
            }
        }
        Ok(())
    }
}
