use anyhow::Result;
use log::{error, info, warn};
use mail_parser::{MessageParser, MimeHeaders};
use rsendmail_i18n::{tr, tr_with_args};
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

/// 从 mail_parser 的地址列表中提取第一个邮箱地址
fn extract_first_email(addrs: Option<&mail_parser::Address>) -> Option<String> {
    addrs.and_then(|addr| {
        addr.iter()
            .find_map(|a| a.address.as_ref().map(|s| s.to_string()))
    })
}

/// 从 mail_parser 的地址列表中提取所有邮箱地址
fn extract_all_emails(addrs: Option<&mail_parser::Address>) -> Vec<String> {
    addrs.map_or_else(Vec::new, |addr| {
        addr.iter()
            .filter_map(|a| a.address.as_ref().map(|s| s.to_string()))
            .collect()
    })
}

/// 从 EML 提取所有 RCPT TO 收件人
/// 如果 include_cc_bcc 为 true，还会提取 Cc 和 Bcc 中的地址（去重）
fn extract_all_recipients(message: &mail_parser::Message, include_cc_bcc: bool) -> Vec<String> {
    let mut recipients = extract_all_emails(message.to());
    if include_cc_bcc {
        recipients.extend(extract_all_emails(message.cc()));
        recipients.extend(extract_all_emails(message.bcc()));
        let mut seen = std::collections::HashSet::new();
        recipients.retain(|addr| seen.insert(addr.to_lowercase()));
    }
    recipients
}

/// 从 config.to 解析全局收件人列表，并过滤空字符串
fn parse_global_recipients(config: &Config) -> Option<Vec<String>> {
    config.to.as_ref()
        .filter(|s| !s.is_empty())
        .map(|to_str| {
            to_str.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        })
}

/// 检测是否为需要重置连接的SMTP错误（基于原始错误字符串）
fn is_connection_error(error: &str) -> bool {
    error.contains("421")
        || error.contains("Cannot accept further commands")
        || error.contains("Broken pipe")
        || error.contains("Connection reset")
        || error.contains("Unparseable SMTP reply")
        || error.contains("timed out")
        || error.contains("timeout")
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
            .unwrap_or_else(|| tr("common.unknown_file"))
    }

    // 保存发送失败的EML文件到指定目录
    fn save_failed_email(config: &Config, source_path: &str) {
        if let Some(ref failed_dir) = config.failed_emails_dir {
            let failed_dir_path = Path::new(failed_dir);

            // 创建目录（如果不存在）
            if let Err(e) = fs::create_dir_all(failed_dir_path) {
                error!(
                    "{}",
                    tr_with_args(
                        "core.mailer.create_failed_dir_error",
                        &[("dir", failed_dir), ("error", &e.to_string())]
                    )
                );
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
                    info!(
                        "{}",
                        tr_with_args(
                            "core.mailer.saved_failed_email",
                            &[("source", source_path), ("dest", &dest_path.display().to_string())]
                        )
                    );
                }
                Err(e) => {
                    error!(
                        "{}",
                        tr_with_args(
                            "core.mailer.save_failed_email_error",
                            &[
                                ("source", source_path),
                                ("dest", &dest_path.display().to_string()),
                                ("error", &e.to_string())
                            ]
                        )
                    );
                }
            }
        }
    }

    pub async fn send_all_with_cancel(&self, running: Arc<AtomicBool>) -> Result<Stats> {
        if let Some(attachment_dir) = &self.config.attachment_dir {
            info!(
                "{}",
                tr_with_args("core.mailer.detecting_attachment_dir", &[("dir", attachment_dir.as_str())])
            );
            return self
                .send_attachment_dir_with_cancel(attachment_dir, running)
                .await;
        }

        if let Some(attachment_path) = &self.config.attachment {
            info!(
                "{}",
                tr_with_args("core.mailer.detecting_attachment", &[("path", attachment_path.as_str())])
            );
            return self
                .send_attachment_with_cancel(attachment_path, running)
                .await;
        }

        let files = self.collect_email_files()?;
        let mut stats = Stats::new();

        match self.config.process_mode() {
            crate::config::ProcessMode::Auto => {
                let num_processes = num_cpus::get();
                info!(
                    "{}",
                    tr_with_args("core.mailer.auto_process_count", &[("count", &num_processes.to_string())])
                );
                self.send_fixed_mode_with_cancel(files, num_processes, &mut stats, running)
                    .await?;
            }
            crate::config::ProcessMode::Fixed(n) => {
                info!(
                    "{}",
                    tr_with_args("core.mailer.using_process_count", &[("count", &n.to_string())])
                );
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
        info!(
            "{}",
            tr_with_args("core.mailer.preparing_attachment_dir", &[("dir", attachment_dir)])
        );
        let mut stats = Stats::new();
        let start = Instant::now();

        let dir_path = Path::new(attachment_dir);
        if !dir_path.exists() || !dir_path.is_dir() {
            let msg = tr_with_args("core.mailer.attachment_dir_not_exist", &[("dir", attachment_dir)]);
            error!("{}", msg);
            return Err(anyhow::anyhow!("{}", msg));
        }

        let mut files = Vec::new();
        info!(
            "{}",
            tr_with_args("core.mailer.scanning_directory", &[("dir", attachment_dir)])
        );
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
        info!(
            "{}",
            tr_with_args("core.mailer.found_files", &[("count", &files.len().to_string())])
        );

        if files.is_empty() {
            info!("{}", tr("core.mailer.directory_empty"));
            stats.total_duration = start.elapsed();
            return Ok(stats);
        }

        info!(
            "{}",
            tr_with_args(
                "core.mailer.connecting_smtp",
                &[("server", &self.config.smtp_server), ("port", &self.config.port.to_string())]
            )
        );

        let use_tls = self.config.use_tls || self.config.port == 465;

        if self.config.auth_mode {
            if let (Some(username), Some(password)) = (&self.config.username, &self.config.password) {
                if use_tls {
                    info!("{}", tr_with_args("core.mailer.using_tls", &[("mode", "auth")]));
                    let mut client_builder =
                        SmtpClientBuilder::new(self.config.smtp_server.as_str(), self.config.port)
                            .credentials((username.as_str(), password.as_str()));
                    client_builder = if self.config.port == 465 {
                        client_builder.implicit_tls(true)
                    } else {
                        client_builder.implicit_tls(false)
                    };
                    if self.config.accept_invalid_certs {
                        client_builder = client_builder.allow_invalid_certs();
                    }
                    match timeout(Duration::from_secs(self.config.smtp_timeout), client_builder.connect()).await {
                        Ok(Ok(mut client)) => {
                            self.send_attachment_dir_files(&files, &mut client, &mut stats, running).await;
                            let _ = client.quit().await;
                        }
                        Ok(Err(e)) => {
                            let msg = tr_with_args("core.mailer.smtp_auth_connect_failed", &[("error", &e.to_string())]);
                            error!("{}", msg);
                            stats.increment_error(&msg, attachment_dir);
                        }
                        Err(_) => {
                            let msg = tr("core.mailer.smtp_auth_timeout");
                            error!("{}", msg);
                            stats.increment_error(&msg, attachment_dir);
                        }
                    }
                } else {
                    let msg = tr("core.mailer.auth_mode_no_tls");
                    error!("{}", msg);
                    stats.increment_error(&msg, attachment_dir);
                }
            } else {
                let msg = tr("core.mailer.auth_mode_missing_credentials");
                error!("{}", msg);
                stats.increment_error(&msg, attachment_dir);
            }
        } else if use_tls {
            info!("{}", tr_with_args("core.mailer.using_tls", &[("mode", "non-auth")]));
            let mut client_builder =
                SmtpClientBuilder::new(self.config.smtp_server.as_str(), self.config.port);
            client_builder = if self.config.port == 465 {
                client_builder.implicit_tls(true)
            } else {
                client_builder.implicit_tls(false)
            };
            if self.config.accept_invalid_certs {
                client_builder = client_builder.allow_invalid_certs();
            }
            match timeout(Duration::from_secs(self.config.smtp_timeout), client_builder.connect()).await {
                Ok(Ok(mut client)) => {
                    self.send_attachment_dir_files(&files, &mut client, &mut stats, running).await;
                    let _ = client.quit().await;
                }
                Ok(Err(e)) => {
                    let msg = tr_with_args("core.mailer.smtp_connect_failed_mode", &[("mode", "non-auth TLS"), ("error", &e.to_string())]);
                    error!("{}", msg);
                    stats.increment_error(&msg, attachment_dir);
                }
                Err(_) => {
                    let msg = tr_with_args("core.mailer.smtp_timeout_mode", &[("mode", "non-auth TLS")]);
                    error!("{}", msg);
                    stats.increment_error(&msg, attachment_dir);
                }
            }
        } else {
            info!("{}", tr_with_args("core.mailer.using_plain", &[("mode", "non-auth")]));
            let client_builder =
                SmtpClientBuilder::new(self.config.smtp_server.as_str(), self.config.port);
            match timeout(Duration::from_secs(self.config.smtp_timeout), client_builder.connect_plain()).await {
                Ok(Ok(mut client)) => {
                    self.send_attachment_dir_files(&files, &mut client, &mut stats, running).await;
                    let _ = client.quit().await;
                }
                Ok(Err(e)) => {
                    let msg = tr_with_args("core.mailer.smtp_connect_failed_mode", &[("mode", "attachment_dir"), ("error", &e.to_string())]);
                    error!("{}", msg);
                    stats.increment_error(&msg, attachment_dir);
                }
                Err(_) => {
                    let msg = tr_with_args("core.mailer.smtp_timeout_mode", &[("mode", "attachment_dir")]);
                    error!("{}", msg);
                    stats.increment_error(&msg, attachment_dir);
                }
            }
        }

        stats.total_duration = start.elapsed();
        Ok(stats)
    }

    /// Generic file-sending loop for attachment-dir mode
    async fn send_attachment_dir_files<T: AsyncRead + AsyncWrite + Unpin + Send>(
        &self,
        files: &[String],
        client: &mut SmtpClient<T>,
        stats: &mut Stats,
        running: Arc<AtomicBool>,
    ) {
        for (file_idx, file_path) in files.iter().enumerate() {
            if !running.load(Ordering::SeqCst) {
                warn!("{}", tr("core.mailer.interrupted"));
                break;
            }

            let send_start = Instant::now();
            let filename = Self::get_filename(file_path);
            let subject = self.config.subject_template.as_ref().map_or_else(
                || format!("Attachment: {}", filename),
                |template| Self::process_template(template, &filename),
            );
            let text_content = self.config.text_template.as_ref().map_or_else(
                || format!("Please find attached: {}", filename),
                |template| Self::process_template(template, &filename),
            );
            let html_content = self
                .config
                .html_template
                .as_ref()
                .map(|template| Self::process_template(template, &filename));

            let empty_params = Parameters::default();
            let from_addr = match self.config.from.as_deref() {
                Some(addr) if !addr.is_empty() => addr,
                _ => {
                    let msg = tr_with_args("core.mailer.set_sender_failed", &[("error", "no sender address specified")]);
                    error!("{}", msg);
                    stats.increment_error(&msg, file_path);
                    continue;
                }
            };
            if let Err(e) = client
                .mail_from(from_addr, &empty_params)
                .await
            {
                let msg = tr_with_args("core.mailer.set_sender_failed", &[("error", &e.to_string())]);
                error!("{}", msg);
                stats.increment_error(&msg, file_path);
                continue;
            }

            let to_str = match self.config.to.as_deref() {
                Some(s) if !s.is_empty() => s,
                _ => {
                    let msg = tr_with_args("core.mailer.all_recipients_failed", &[("path", file_path)]);
                    error!("{}", msg);
                    stats.increment_error(&msg, file_path);
                    let _ = client.rset().await; // RSET after incomplete MAIL FROM transaction
                    continue;
                }
            };
            let recipients: Vec<&str> = to_str
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();

            if recipients.is_empty() {
                let msg = tr_with_args(
                    "core.mailer.set_recipient_failed_for",
                    &[("recipient", to_str), ("path", file_path), ("error", "empty")]
                );
                error!("{}", msg);
                stats.increment_error(&tr("core.mailer.all_recipients_failed"), file_path);
                let _ = client.rset().await; // RSET after incomplete MAIL FROM transaction
                continue;
            }

            let mut any_rcpt_succeeded = false;
            for recipient in &recipients {
                if let Err(e) = client.rcpt_to(recipient, &empty_params).await {
                    let msg = tr_with_args(
                        "core.mailer.set_recipient_failed_for",
                        &[("recipient", *recipient), ("path", file_path), ("error", &e.to_string())]
                    );
                    error!("{}", msg);
                    stats.increment_error(&msg, file_path);
                } else {
                    info!(
                        "{}",
                        tr_with_args(
                            "core.mailer.set_recipient_success",
                            &[("recipient", *recipient), ("path", file_path)]
                        )
                    );
                    any_rcpt_succeeded = true;
                }
            }

            if !any_rcpt_succeeded {
                let msg = tr_with_args("core.mailer.all_recipients_failed", &[("path", file_path)]);
                error!("{}", msg);
                let _ = client.rset().await; // RSET after incomplete transaction
                continue;
            }

            let attachment_content = match tokio::fs::read(file_path).await {
                Ok(content) => content,
                Err(e) => {
                    let msg = tr_with_args("core.mailer.read_attachment_failed", &[("error", &e.to_string())]);
                    error!("{}", msg);
                    stats.increment_error(&msg, file_path);
                    continue;
                }
            };

            let mut builder = MessageBuilder::new()
                .from(("", from_addr))
                .to(recipients)
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
                    let msg = tr_with_args("core.mailer.build_email_failed", &[("error", &e.to_string())]);
                    error!("{}", msg);
                    stats.increment_error(&msg, file_path);
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
                    info!(
                        "{}",
                        tr_with_args("core.mailer.attachment_email_success", &[("file", &filename)])
                    );
                    stats.email_count += 1;
                    stats.send_durations.push(send_start.elapsed());
                }
                Ok(Err(e)) => {
                    let msg = tr_with_args(
                        "core.mailer.email_send_failed_for",
                        &[("path", file_path), ("error", &e.to_string())]
                    );
                    error!("{}", msg);
                    stats.increment_error(&msg, file_path);
                }
                Err(_) => {
                    let msg = tr_with_args("core.mailer.email_send_timeout_for", &[("path", file_path)]);
                    error!("{}", msg);
                    stats.increment_error(&tr("core.mailer.email_send_timeout"), file_path);
                }
            }

            if self.config.email_send_interval_ms > 0
                && (file_idx + 1 < files.len())
                && running.load(Ordering::SeqCst)
            {
                info!(
                    "{}",
                    tr_with_args(
                        "core.mailer.waiting_next_batch",
                        &[("current", &(file_idx + 1).to_string()), ("total", &files.len().to_string())]
                    )
                );
                let sleep_duration =
                    std::time::Duration::from_millis(self.config.email_send_interval_ms);
                let running_clone_for_sleep = running.clone();
                tokio::select! {
                    biased;
                    _ = async { loop { if !running_clone_for_sleep.load(Ordering::SeqCst) { break; } tokio::time::sleep(Duration::from_millis(100)).await; } } => {
                        warn!(
                            "{}",
                            tr_with_args(
                                "core.mailer.attachment_dir_interval_interrupted",
                                &[("current", &(file_idx + 1).to_string()), ("total", &files.len().to_string())]
                            )
                        );
                    }
                    _ = tokio::time::sleep(sleep_duration) => {}
                }
                if !running.load(Ordering::SeqCst) {
                    warn!("{}", tr("core.mailer.interrupted"));
                    break;
                }
            }
        }
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
            warn!("{}", tr("core.mailer.execute_send_interrupted"));
            return Ok(()); // Not an error, but operation stopped
        }

        let send_start = Instant::now();
        let empty_params = Parameters::default();

        let from_addr = match self.config.from.as_deref() {
            Some(addr) if !addr.is_empty() => addr,
            _ => {
                let msg = tr_with_args(
                    "core.mailer.set_sender_failed_for",
                    &[("path", attachment_path), ("error", "no sender address specified")]
                );
                error!("{}", msg);
                stats.increment_error(&msg, attachment_path);
                return Ok(());
            }
        };
        if let Err(e) = client
            .mail_from(from_addr, &empty_params)
            .await
        {
            let msg = tr_with_args(
                "core.mailer.set_sender_failed_for",
                &[("path", attachment_path), ("error", &e.to_string())]
            );
            error!("{}", msg);
            stats.increment_error(&msg, attachment_path);
            return Ok(());
        }

        let to_str = match self.config.to.as_deref() {
            Some(s) if !s.is_empty() => s,
            _ => {
                let msg = tr_with_args(
                    "core.mailer.all_recipients_failed",
                    &[("path", attachment_path)]
                );
                error!("{}", msg);
                stats.increment_error(&msg, attachment_path);
                return Ok(());
            }
        };
        let recipients: Vec<&str> = to_str
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        if recipients.is_empty() {
            let msg = tr_with_args(
                "core.mailer.all_recipients_failed",
                &[("path", attachment_path)]
            );
            error!("{}", msg);
            stats.increment_error(&msg, attachment_path);
            return Ok(());
        }

        let mut any_rcpt_succeeded = false;
        for recipient in &recipients {
            if let Err(e) = client.rcpt_to(recipient, &empty_params).await {
                let msg = tr_with_args(
                    "core.mailer.set_recipient_failed_for",
                    &[("recipient", *recipient), ("path", attachment_path), ("error", &e.to_string())]
                );
                error!("{}", msg);
                stats.increment_error(&msg, attachment_path);
            } else {
                info!(
                    "{}",
                    tr_with_args(
                        "core.mailer.set_recipient_success",
                        &[("recipient", *recipient), ("path", attachment_path)]
                    )
                );
                any_rcpt_succeeded = true;
            }
        }

        if !any_rcpt_succeeded {
            let msg = tr_with_args("core.mailer.all_recipients_failed", &[("path", attachment_path)]);
            error!("{}", msg);
            // increment_error is already done per recipient
            return Ok(());
        }

        let attachment_content = match tokio::fs::read(attachment_path).await {
            Ok(content) => content,
            Err(e) => {
                let msg = tr_with_args(
                    "core.mailer.read_attachment_failed_for",
                    &[("path", attachment_path), ("error", &e.to_string())]
                );
                error!("{}", msg);
                stats.increment_error(&msg, attachment_path);
                return Ok(());
            }
        };

        let mut builder = MessageBuilder::new()
            .from(("", from_addr))
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
                let msg = tr_with_args(
                    "core.mailer.build_email_failed_for",
                    &[("path", attachment_path), ("error", &e.to_string())]
                );
                error!("{}", msg);
                stats.increment_error(&msg, attachment_path);
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
                info!(
                    "{}",
                    tr_with_args("core.mailer.attachment_email_success_path", &[("path", attachment_path)])
                );
                stats.email_count += 1;
                stats.send_durations.push(send_start.elapsed());
            }
            Ok(Err(e)) => {
                let msg = tr_with_args(
                    "core.mailer.email_send_failed_for",
                    &[("path", attachment_path), ("error", &e.to_string())]
                );
                error!("{}", msg);
                stats.increment_error(&msg, attachment_path);
            }
            Err(_) => {
                let msg = tr_with_args("core.mailer.email_send_timeout_for", &[("path", attachment_path)]);
                error!("{}", msg);
                stats.increment_error(&tr("core.mailer.email_send_timeout"), attachment_path);
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
        info!(
            "{}",
            tr_with_args("core.mailer.preparing_attachment", &[("path", attachment_path)])
        );
        let mut stats = Stats::new();
        let start = Instant::now();

        if !Path::new(attachment_path).exists() {
            let msg = tr_with_args("core.mailer.attachment_not_exist", &[("path", attachment_path)]);
            error!("{}", msg);
            stats.increment_error(&msg, attachment_path); // Record error in stats
            return Ok(stats); // Return stats with error instead of Err(anyhow!)
        }

        let filename = Self::get_filename(attachment_path);
        let subject = self.config.subject_template.as_ref().map_or_else(
            || format!("Attachment: {}", filename),
            |template| Self::process_template(template, &filename),
        );
        let text_content = self.config.text_template.as_ref().map_or_else(
            || format!("Please find attached: {}", filename),
            |template| Self::process_template(template, &filename),
        );
        let html_content = self
            .config
            .html_template
            .as_ref()
            .map(|template| Self::process_template(template, &filename));

        info!(
            "{}",
            tr_with_args(
                "core.mailer.connecting_smtp",
                &[("server", &self.config.smtp_server), ("port", &self.config.port.to_string())]
            )
        );
        let use_tls = self.config.use_tls || self.config.port == 465;

        // No longer need client_result to be a single variable for different types.
        // We will handle connection and then call execute_send_logic within each branch.

        if self.config.auth_mode {
            if let (Some(username), Some(password)) = (&self.config.username, &self.config.password)
            {
                info!(
                    "{}",
                    tr_with_args("core.mailer.using_account_login", &[("username", username.as_str())])
                );
                if use_tls {
                    info!("{}", tr_with_args("core.mailer.using_tls", &[("mode", "auth")]));
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
                            let msg = tr_with_args("core.mailer.smtp_auth_connect_failed", &[("error", &e.to_string())]);
                            error!("{}", msg);
                            stats.increment_error(&msg, attachment_path);
                        }
                        Err(_) => {
                            let msg = tr("core.mailer.smtp_auth_timeout");
                            error!("{}", msg);
                            stats.increment_error(&msg, attachment_path);
                        }
                    }
                } else {
                    let msg = tr("core.mailer.auth_mode_no_tls");
                    error!("{}", msg);
                    stats.increment_error(&msg, attachment_path);
                }
            } else {
                let msg = tr("core.mailer.auth_mode_missing_credentials");
                error!("{}", msg);
                stats.increment_error(&msg, attachment_path);
            }
        } else {
            // Non-authenticated mode
            let mut client_builder =
                SmtpClientBuilder::new(self.config.smtp_server.as_str(), self.config.port);
            if use_tls {
                info!("{}", tr_with_args("core.mailer.using_tls", &[("mode", "non-auth")]));
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
                        let msg = tr_with_args(
                            "core.mailer.smtp_connect_failed_mode",
                            &[("mode", "non-auth TLS"), ("error", &e.to_string())]
                        );
                        error!("{}", msg);
                        stats.increment_error(
                            &msg,
                            attachment_path,
                        );
                    }
                    Err(_) => {
                        let msg = tr_with_args("core.mailer.smtp_timeout_mode", &[("mode", "non-auth TLS")]);
                        error!("{}", msg);
                        stats.increment_error(&msg, attachment_path);
                    }
                }
            } else {
                // Plain connection
                info!("{}", tr_with_args("core.mailer.using_plain", &[("mode", "non-auth")]));
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
                        let msg = tr_with_args(
                            "core.mailer.smtp_connect_failed_mode",
                            &[("mode", "non-auth Plain"), ("error", &e.to_string())]
                        );
                        error!("{}", msg);
                        stats.increment_error(&msg, attachment_path);
                    }
                    Err(_) => {
                        let msg = tr_with_args("core.mailer.smtp_timeout_mode", &[("mode", "non-auth Plain")]);
                        error!("{}", msg);
                        stats.increment_error(&msg, attachment_path);
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
        if files.is_empty() {
            info!("{}", tr("core.mailer.directory_empty"));
            return Ok(());
        }
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
                        warn!(
                            "{}",
                            tr_with_args("core.mailer.process_group_interrupted", &[("id", &(i + 1).to_string())])
                        );
                        break;
                    }

                    current_batch.push(file.clone());

                    if current_batch.len() >= config.batch_size || j + 1 == chunk.len() {
                        info!(
                            "{}",
                            tr_with_args(
                                "core.mailer.process_group_sending",
                                &[
                                    ("id", &(i + 1).to_string()),
                                    ("current", &(j / config.batch_size + 1).to_string()),
                                    ("total", &chunk.len().div_ceil(config.batch_size).to_string()),
                                    ("file", &current_batch.len().to_string())
                                ]
                            )
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
                                                error!("{}", tr_with_args("core.mailer.process_group_tls_failed", &[("id", &(i + 1).to_string()), ("error", &e.to_string())]));
                                                for file_path_in_batch in &current_batch {
                                                    group_stats.3.push((
                                                        tr_with_args("core.mailer.error_tls_batch", &[("error", &e.to_string())]),
                                                        file_path_in_batch.clone(),
                                                    ));
                                                }
                                            }
                                            let _ = client.quit().await;
                                        }
                                        Ok(Err(e)) => {
                                            error!("{}", tr_with_args("core.mailer.process_group_auth_failed", &[("id", &(i + 1).to_string()), ("error", &e.to_string())]));
                                            for file_path_in_batch in &current_batch {
                                                group_stats.3.push((
                                                    tr("core.mailer.error_auth_connect_failed"),
                                                    file_path_in_batch.clone(),
                                                ));
                                            }
                                        }
                                        Err(_) => {
                                            error!("{}", tr_with_args("core.mailer.process_group_auth_timeout", &[("id", &(i + 1).to_string())]));
                                            for file_path_in_batch in &current_batch {
                                                group_stats.3.push((
                                                    tr("core.mailer.error_auth_connect_timeout"),
                                                    file_path_in_batch.clone(),
                                                ));
                                            }
                                        }
                                    }
                                } else {
                                    error!("{}", tr_with_args("core.mailer.process_group_no_tls_auth", &[("id", &(i + 1).to_string())]));
                                    for file_path_in_batch in &current_batch {
                                        group_stats.3.push((
                                            tr("core.mailer.error_auth_requires_tls"),
                                            file_path_in_batch.clone(),
                                        ));
                                    }
                                }
                            } else {
                                error!("{}", tr_with_args("core.mailer.process_group_missing_auth", &[("id", &(i + 1).to_string())]));
                                for file_path_in_batch in &current_batch {
                                    group_stats.3.push((
                                        tr("core.mailer.error_auth_missing_credentials"),
                                        file_path_in_batch.clone(),
                                    ));
                                }
                            }
                        } else {
                            // Non-authenticated mode
                            if use_tls {
                                // Non-auth + TLS: no client_opt reuse, new connection per batch
                                client_opt = None;
                                info!("{}", tr_with_args("core.mailer.process_group_using_tls", &[("id", &(i + 1).to_string())]));
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
                                            error!("{}", tr_with_args("core.mailer.process_group_tls_failed", &[("id", &(i + 1).to_string()), ("error", &e.to_string())]));
                                            for file_path_in_batch in &current_batch {
                                                group_stats.3.push((
                                                    tr_with_args("core.mailer.error_non_auth_tls_batch", &[("error", &e.to_string())]),
                                                    file_path_in_batch.clone(),
                                                ));
                                            }
                                        }
                                        let _ = client.quit().await;
                                    }
                                    Ok(Err(e)) => {
                                        error!("{}", tr_with_args("core.mailer.process_group_non_auth_tls_failed", &[("id", &(i + 1).to_string()), ("error", &e.to_string())]));
                                        for file_path_in_batch in &current_batch {
                                            group_stats.3.push((
                                                tr("core.mailer.error_non_auth_tls_failed"),
                                                file_path_in_batch.clone(),
                                            ));
                                        }
                                    }
                                    Err(_) => {
                                        error!("{}", tr_with_args("core.mailer.process_group_non_auth_tls_timeout", &[("id", &(i + 1).to_string())]));
                                        for file_path_in_batch in &current_batch {
                                            group_stats.3.push((
                                                tr("core.mailer.error_non_auth_tls_timeout"),
                                                file_path_in_batch.clone(),
                                            ));
                                        }
                                    }
                                }
                            } else {
                                // Non-auth + Plain: use client_opt for potential reuse
                                if client_opt.is_none() {
                                    info!("{}", tr_with_args("core.mailer.process_group_using_plain", &[("id", &(i + 1).to_string()), ("batch", &config.batch_size.to_string())]));
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
                                            error!("{}", tr_with_args("core.mailer.process_group_plain_connect_failed", &[("id", &(i + 1).to_string()), ("error", &e.to_string())]));
                                            for file_path_in_batch in &current_batch {
                                                group_stats.3.push((
                                                    tr("core.mailer.error_plain_connect_failed"),
                                                    file_path_in_batch.clone(),
                                                ));
                                            }
                                        }
                                        Err(_) => {
                                            error!("{}", tr_with_args("core.mailer.process_group_plain_timeout", &[("id", &(i + 1).to_string())]));
                                            for file_path_in_batch in &current_batch {
                                                group_stats.3.push((
                                                    tr("core.mailer.error_plain_timeout"),
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
                                        warn!("{}", tr_with_args("core.mailer.process_group_connection_reset_needed", &[("id", &(i + 1).to_string())]));
                                        // 立即重置连接，下个批次将重新建立
                                        client_opt = None;
                                    }

                                    // batch-size=1时强制关闭连接，避免连接重用
                                    if config.batch_size == 1 {
                                        info!("{}", tr_with_args("core.mailer.process_group_batch_size_1_close", &[("id", &(i + 1).to_string())]));
                                        client_opt = None;
                                    }
                                } else {
                                    info!("{}", tr_with_args("core.mailer.process_group_smtp_unavailable", &[("id", &(i + 1).to_string())]));
                                }
                            }
                        }
                        current_batch.clear();
                        if config.email_send_interval_ms > 0
                            && j + 1 < chunk.len()
                            && running.load(Ordering::SeqCst)
                        {
                            info!("{}", tr_with_args("core.mailer.process_group_batch_interval", &[("id", &(i + 1).to_string()), ("ms", &config.email_send_interval_ms.to_string()), ("current", &(j + 1).to_string()), ("total", &chunk.len().to_string())]));
                            let sleep_duration =
                                std::time::Duration::from_millis(config.email_send_interval_ms);
                            let running_clone_for_sleep = running.clone();
                            tokio::select! {
                                biased;
                                _ = async { loop { if !running_clone_for_sleep.load(Ordering::SeqCst) { break; } tokio::time::sleep(Duration::from_millis(100)).await; } } => { warn!("{}", tr_with_args("core.mailer.task_interval_interrupted", &[("id", &(i + 1).to_string()), ("current", &(j + 1).to_string()), ("total", &chunk.len().to_string())])); }
                                _ = tokio::time::sleep(sleep_duration) => {}
                            }
                            if !running.load(Ordering::SeqCst) {
                                warn!("{}", tr_with_args("core.mailer.process_group_interval_interrupted", &[("id", &(i + 1).to_string()), ("current", &(j + 1).to_string()), ("total", &chunk.len().to_string())]));
                                break;
                            }
                        }
                    }
                }
                info!(
                    "{}",
                    tr_with_args("core.mailer.process_group_complete", &[("id", &(i + 1).to_string())])
                );
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
                info!("{}", tr("core.mailer.using_attachment_mode"));
                return Ok(files);
            }
        };
        info!(
            "{}",
            tr_with_args("core.mailer.scanning_eml_directory", &[("dir", dir.as_str())])
        );
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
        info!(
            "{}",
            tr_with_args("core.mailer.found_eml_files", &[("count", &files.len().to_string())])
        );
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

        // 构建全局收件人列表（如果CLI指定了--to）
        let global_recipients = parse_global_recipients(config);

        for (email_idx, file_path) in files.iter().enumerate() {
            if !running.load(Ordering::SeqCst) {
                warn!("{}", tr("core.mailer.send_batch_interrupted"));
                break;
            }
            let mut had_error_this_email = false;
            let mut current_file_parse_duration: Option<Duration> = None;
            let parse_start = Instant::now();

            let content_read_result = tokio::fs::read(file_path).await;

            let content = match content_read_result {
                Ok(c) => {
                    current_file_parse_duration = Some(parse_start.elapsed());
                    if let Some(anonymizer_ref) = anonymizer.as_mut() {
                        info!("{}", tr_with_args("core.mailer.anonymizing_email", &[("path", file_path.as_str())]));
                        anonymizer_ref.anonymize_binary(&c)
                    } else {
                        c
                    }
                }
                Err(e) => {
                    error!("{}", tr_with_args("core.mailer.read_file_failed", &[("path", file_path.as_str()), ("error", &e.to_string())]));
                    failures.push((tr_with_args("core.mailer.error_read_file", &[("error", &e.to_string())]), file_path.to_string()));
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
                        error!("{}", tr_with_args("core.mailer.parse_email_failed", &[("path", file_path.as_str())]));
                        failures.push((tr("core.mailer.error_parse_email"), file_path.to_string()));
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

                    // 确定发件人地址：优先使用CLI指定的--from，否则从EML提取
                    let envelope_from = if let Some(ref from) = config.from.as_ref().filter(|s| !s.is_empty()) {
                        from.to_string()
                    } else {
                        match extract_first_email(message.from()) {
                            Some(addr) => {
                                info!("{}", tr_with_args("core.mailer.using_eml_from", &[("addr", addr.as_str()), ("path", file_path.as_str())]));
                                addr
                            }
                            None => {
                                error!("{}", tr_with_args("core.mailer.extract_from_failed", &[("path", file_path.as_str())]));
                                failures.push((tr("core.mailer.error_no_from"), file_path.to_string()));
                                Self::save_failed_email(config, file_path);
                                continue;
                            }
                        }
                    };

                    // 确定收件人地址：优先使用CLI指定的--to，否则从EML提取
                    let current_recipients: Vec<String> = if let Some(ref recips) = global_recipients {
                        recips.clone()
                    } else {
                        let eml_recipients = extract_all_recipients(&message, config.envelope_cc_bcc);
                        if !eml_recipients.is_empty() {
                            info!("{}", tr_with_args("core.mailer.using_eml_recipients", &[("addrs", &format!("{:?}", eml_recipients)), ("path", file_path.as_str())]));
                        }
                        eml_recipients
                    };

                    if current_recipients.is_empty() {
                        error!("{}", tr_with_args("core.mailer.no_valid_recipients", &[("path", file_path.as_str()), ("to", config.to.as_deref().unwrap_or("<from EML>"))]));
                        failures.push((
                            tr_with_args("core.mailer.error_no_recipients", &[("to", config.to.as_deref().unwrap_or("<from EML>"))]),
                            file_path.to_string(),
                        ));
                        Self::save_failed_email(config, file_path);
                        email_send_op_failed = true;
                    }

                    if !email_send_op_failed {
                        if let Err(e) = client.mail_from(&envelope_from, &empty_params).await
                        {
                            let raw_error = e.to_string();
                            error!("{}", tr_with_args("core.mailer.set_sender_failed_for", &[("path", file_path.as_str()), ("error", &raw_error)]));
                            let error_msg = tr_with_args("core.mailer.error_set_sender", &[("error", &raw_error)]);
                            failures.push((error_msg.clone(), file_path.to_string()));
                            Self::save_failed_email(config, file_path);
                            email_send_op_failed = true;

                            // 检测关键SMTP错误（基于原始错误字符串，非翻译文本）
                            if is_connection_error(&raw_error) {
                                warn!("{}", tr_with_args("core.mailer.connection_error_detected", &[("error", &raw_error)]));
                                connection_should_reset = true;
                                break;
                            }
                        }
                    }

                    if !email_send_op_failed {
                        let mut any_rcpt_succeeded = false;
                        for recipient in &current_recipients {
                            if let Err(e) = client.rcpt_to(recipient.as_str(), &empty_params).await {
                                error!("{}", tr_with_args("core.mailer.set_recipient_failed_for", &[("recipient", recipient.as_str()), ("path", file_path.as_str()), ("error", &e.to_string())]));
                                failures.push((
                                    tr_with_args("core.mailer.error_set_recipient", &[("recipient", recipient.as_str()), ("error", &e.to_string())]),
                                    file_path.to_string(),
                                ));
                            } else {
                                info!("{}", tr_with_args("core.mailer.set_recipient_success", &[("recipient", recipient.as_str()), ("path", file_path.as_str())]));
                                any_rcpt_succeeded = true;
                            }
                        }
                        if !any_rcpt_succeeded && !current_recipients.is_empty() {
                            error!("{}", tr_with_args("core.mailer.all_recipients_failed", &[("path", file_path.as_str())]));
                            Self::save_failed_email(config, file_path);
                            email_send_op_failed = true;
                        }
                    }

                    if !email_send_op_failed {
                        let mail_data_to_send = if config.keep_headers {
                            info!("{}", tr_with_args("core.mailer.using_original_headers", &[("path", file_path.as_str())]));
                            content.clone()
                        } else if config.modify_headers {
                            info!("{}", tr_with_args("core.mailer.modifying_headers", &[("path", file_path.as_str())]));
                            let subject = message.subject().unwrap_or("No Subject").to_string();
                            let text_content = message.body_text(0).unwrap_or_default().to_string();
                            let html_content = message.body_html(0).map(|s| s.to_string());
                            let recipients_str: Vec<&str> = current_recipients.iter().map(|s| s.as_str()).collect();
                            let mut builder = MessageBuilder::new()
                                .from(("", envelope_from.as_str()))
                                .to(recipients_str)
                                .subject(&subject)
                                .text_body(&text_content);
                            if let Some(html) = &html_content {
                                builder = builder.html_body(html);
                            }
                            // 保留原始附件
                            for att_part in message.attachments() {
                                let att_name = att_part.attachment_name().unwrap_or("attachment");
                                let att_type = att_part.content_type()
                                    .map(|ct: &mail_parser::ContentType| ct.ctype())
                                    .unwrap_or("application/octet-stream");
                                builder = builder.attachment(att_type, att_name, att_part.contents());
                            }
                            match builder.write_to_vec() {
                                Ok(m_content) => m_content,
                                Err(e) => {
                                    error!("{}", tr_with_args("core.mailer.build_email_failed_for", &[("path", file_path.as_str()), ("error", &e.to_string())]));
                                    failures.push((
                                        tr_with_args("core.mailer.error_build_email", &[("error", &e.to_string())]),
                                        file_path.to_string(),
                                    ));
                                    Self::save_failed_email(config, file_path);
                                    email_send_op_failed = true;
                                    Vec::new()
                                }
                            }
                        } else {
                            // 在默认模式下使用原始邮件内容来保持附件和完整的MIME结构
                            info!("{}", tr_with_args("core.mailer.using_original_content", &[("path", file_path.as_str())]));
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
                                    info!("{}", tr_with_args("core.mailer.email_send_success", &[("path", file_path.as_str())]));
                                    successes.push((parse_duration_final, send_start.elapsed()));
                                }
                                Ok(Err(e)) => {
                                    let raw_error = e.to_string();
                                    error!("{}", tr_with_args("core.mailer.email_send_failed_for", &[("path", file_path.as_str()), ("error", &raw_error)]));
                                    let error_msg = tr_with_args("core.mailer.error_send_failed", &[("error", &raw_error)]);
                                    failures.push((error_msg, file_path.to_string()));
                                    Self::save_failed_email(config, file_path);

                                    // 检测关键SMTP错误（基于原始错误字符串）
                                    if is_connection_error(&raw_error) {
                                        warn!("{}", tr_with_args("core.mailer.data_connection_error", &[("error", &raw_error)]));
                                        connection_should_reset = true;
                                        break;
                                    }
                                }
                                Err(_) => {
                                    error!("{}", tr_with_args("core.mailer.email_send_timeout_for", &[("path", file_path.as_str())]));
                                    failures
                                        .push((tr("core.mailer.error_send_timeout"), file_path.to_string()));
                                    Self::save_failed_email(config, file_path);
                                }
                            }
                        }
                    }
                }
            }

            // 添加RSET命令：如果还有更多邮件要发送，重置SMTP状态
            if email_idx + 1 < files.len()
                && running.load(Ordering::SeqCst)
                && !connection_should_reset
            {
                info!("{}", tr_with_args("core.mailer.rset_command", &[("current", &(email_idx + 1).to_string()), ("total", &files.len().to_string())]));
                if let Err(e) = client.rset().await {
                    warn!("{}", tr_with_args("core.mailer.rset_command_failed", &[("current", &(email_idx + 1).to_string()), ("total", &files.len().to_string()), ("error", &e.to_string())]));
                    // RSET失败通常意味着连接有问题，标记需要重置连接
                    connection_should_reset = true;
                    break;
                }
            }

            if config.email_send_interval_ms > 0
                && email_idx + 1 < files.len()
                && running.load(Ordering::SeqCst)
            {
                info!("{}", tr_with_args("core.mailer.waiting_next_email", &[("ms", &config.email_send_interval_ms.to_string()), ("current", &(email_idx + 1).to_string()), ("total", &files.len().to_string())]));
                let sleep_duration =
                    std::time::Duration::from_millis(config.email_send_interval_ms);
                let running_clone_for_sleep = running.clone();
                tokio::select! {
                    biased;
                    _ = async { loop { if !running_clone_for_sleep.load(Ordering::SeqCst) { break; } tokio::time::sleep(Duration::from_millis(100)).await; } } => {
                        warn!("{}", tr_with_args("core.mailer.email_interval_interrupted", &[("current", &(email_idx + 1).to_string()), ("total", &files.len().to_string())]));
                    }
                    _ = tokio::time::sleep(sleep_duration) => {}
                }
                if !running.load(Ordering::SeqCst) {
                    warn!("{}", tr_with_args("core.mailer.email_interval_exit", &[("current", &(email_idx + 1).to_string()), ("total", &files.len().to_string())]));
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

        // 构建全局收件人列表（如果CLI指定了--to）
        let global_recipients = parse_global_recipients(config);

        for (email_idx, file_path) in files.iter().enumerate() {
            if !running.load(Ordering::SeqCst) {
                warn!("{}", tr_with_args("core.mailer.process_group_interrupted", &[("id", &process_group_id.to_string())]));
                break;
            }
            let mut had_error_this_email = false;
            let mut current_file_parse_duration: Option<Duration> = None;
            let parse_start = Instant::now();

            let content_read_result = tokio::fs::read(file_path).await;

            let content = match content_read_result {
                Ok(c) => {
                    current_file_parse_duration = Some(parse_start.elapsed());
                    if let Some(anonymizer_ref) = anonymizer.as_mut() {
                        info!("{}", tr_with_args("core.mailer.anonymizing_email", &[("path", file_path.as_str())]));
                        anonymizer_ref.anonymize_binary(&c)
                    } else {
                        c
                    }
                }
                Err(e) => {
                    error!("{}", tr_with_args("core.mailer.read_file_failed", &[("path", file_path.as_str()), ("error", &e.to_string())]));
                    group_stats
                        .3
                        .push((tr_with_args("core.mailer.error_read_file", &[("error", &e.to_string())]), file_path.to_string()));
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
                        error!("{}", tr_with_args("core.mailer.parse_email_failed", &[("path", file_path.as_str())]));
                        group_stats
                            .3
                            .push((tr("core.mailer.error_parse_email"), file_path.to_string()));
                        Self::save_failed_email(config, file_path);
                        had_error_this_email = true;
                        MessageParser::default().parse(b"Subject: error").unwrap()
                    }
                };

                if !had_error_this_email {
                    let send_start = Instant::now();
                    let empty_params = Parameters::default();
                    let mut email_send_op_failed = false;

                    // 确定发件人地址：优先使用CLI指定的--from，否则从EML提取
                    let envelope_from = if let Some(ref from) = config.from.as_ref().filter(|s| !s.is_empty()) {
                        from.to_string()
                    } else {
                        match extract_first_email(message.from()) {
                            Some(addr) => {
                                info!("{}", tr_with_args("core.mailer.using_eml_from", &[("addr", addr.as_str()), ("path", file_path.as_str())]));
                                addr
                            }
                            None => {
                                error!("{}", tr_with_args("core.mailer.extract_from_failed", &[("path", file_path.as_str())]));
                                group_stats.3.push((tr("core.mailer.error_no_from"), file_path.to_string()));
                                Self::save_failed_email(config, file_path);
                                continue;
                            }
                        }
                    };

                    // 确定收件人地址：优先使用CLI指定的--to，否则从EML提取
                    let current_recipients: Vec<String> = if let Some(ref recips) = global_recipients {
                        recips.clone()
                    } else {
                        let eml_recipients = extract_all_recipients(&message, config.envelope_cc_bcc);
                        if !eml_recipients.is_empty() {
                            info!("{}", tr_with_args("core.mailer.using_eml_recipients", &[("addrs", &format!("{:?}", eml_recipients)), ("path", file_path.as_str())]));
                        }
                        eml_recipients
                    };

                    if current_recipients.is_empty() {
                        error!("{}", tr_with_args("core.mailer.no_valid_recipients", &[("path", file_path.as_str()), ("to", config.to.as_deref().unwrap_or("<from EML>"))]));
                        group_stats.3.push((
                            tr_with_args("core.mailer.error_no_recipients", &[("to", config.to.as_deref().unwrap_or("<from EML>"))]),
                            file_path.to_string(),
                        ));
                        Self::save_failed_email(config, file_path);
                        email_send_op_failed = true;
                    }

                    if !email_send_op_failed {
                        if let Err(e) = client.mail_from(&envelope_from, &empty_params).await
                        {
                            let raw_error = e.to_string();
                            error!("{}", tr_with_args("core.mailer.set_sender_failed_for", &[("path", file_path.as_str()), ("error", &raw_error)]));
                            let error_msg = tr_with_args("core.mailer.error_set_sender", &[("error", &raw_error)]);
                            group_stats
                                .3
                                .push((error_msg, file_path.to_string()));
                            Self::save_failed_email(config, file_path);
                            email_send_op_failed = true;

                            // 检测关键SMTP错误（基于原始错误字符串）
                            if is_connection_error(&raw_error) {
                                warn!("{}", tr_with_args("core.mailer.connection_error_detected", &[("error", &raw_error)]));
                                break;
                            }
                        }
                    }

                    if !email_send_op_failed {
                        let mut any_rcpt_succeeded = false;
                        for recipient in &current_recipients {
                            if let Err(e) = client.rcpt_to(recipient.as_str(), &empty_params).await {
                                error!("{}", tr_with_args("core.mailer.set_recipient_failed_for", &[("recipient", recipient.as_str()), ("path", file_path.as_str()), ("error", &e.to_string())]));
                                group_stats.3.push((
                                    tr_with_args("core.mailer.error_set_recipient", &[("recipient", recipient.as_str()), ("error", &e.to_string())]),
                                    file_path.to_string(),
                                ));
                            } else {
                                info!("{}", tr_with_args("core.mailer.set_recipient_success", &[("recipient", recipient.as_str()), ("path", file_path.as_str())]));
                                any_rcpt_succeeded = true;
                            }
                        }
                        if !any_rcpt_succeeded && !current_recipients.is_empty() {
                            error!("{}", tr_with_args("core.mailer.all_recipients_failed", &[("path", file_path.as_str())]));
                            Self::save_failed_email(config, file_path);
                            email_send_op_failed = true;
                        }
                    }

                    if !email_send_op_failed {
                        let mail_data_to_send = if config.keep_headers {
                            info!("{}", tr_with_args("core.mailer.using_original_headers", &[("path", file_path.as_str())]));
                            content.clone()
                        } else if config.modify_headers {
                            info!("{}", tr_with_args("core.mailer.modifying_headers", &[("path", file_path.as_str())]));
                            let subject = message.subject().unwrap_or("No Subject").to_string();
                            let text_content = message.body_text(0).unwrap_or_default().to_string();
                            let html_content = message.body_html(0).map(|s| s.to_string());
                            let recipients_str: Vec<&str> = current_recipients.iter().map(|s| s.as_str()).collect();
                            let mut builder = MessageBuilder::new()
                                .from(("", envelope_from.as_str()))
                                .to(recipients_str)
                                .subject(&subject)
                                .text_body(&text_content);
                            if let Some(html) = &html_content {
                                builder = builder.html_body(html);
                            }
                            // 保留原始附件
                            for att_part in message.attachments() {
                                let att_name = att_part.attachment_name().unwrap_or("attachment");
                                let att_type = att_part.content_type()
                                    .map(|ct: &mail_parser::ContentType| ct.ctype())
                                    .unwrap_or("application/octet-stream");
                                builder = builder.attachment(att_type, att_name, att_part.contents());
                            }
                            match builder.write_to_vec() {
                                Ok(m_content) => m_content,
                                Err(e) => {
                                    error!("{}", tr_with_args("core.mailer.build_email_failed_for", &[("path", file_path.as_str()), ("error", &e.to_string())]));
                                    group_stats.3.push((
                                        tr_with_args("core.mailer.error_build_email", &[("error", &e.to_string())]),
                                        file_path.to_string(),
                                    ));
                                    Self::save_failed_email(config, file_path);
                                    email_send_op_failed = true;
                                    Vec::new()
                                }
                            }
                        } else {
                            // 在默认模式下使用原始邮件内容来保持附件和完整的MIME结构
                            info!("{}", tr_with_args("core.mailer.using_original_content", &[("path", file_path.as_str())]));
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
                                    info!("{}", tr_with_args("core.mailer.email_send_success", &[("path", file_path.as_str())]));
                                    group_stats.0 += 1;
                                    group_stats.1.push(parse_duration_final);
                                    group_stats.2.push(send_start.elapsed());
                                }
                                Ok(Err(e)) => {
                                    let raw_error = e.to_string();
                                    error!("{}", tr_with_args("core.mailer.email_send_failed_for", &[("path", file_path.as_str()), ("error", &raw_error)]));
                                    let error_msg = tr_with_args("core.mailer.error_send_failed", &[("error", &raw_error)]);
                                    group_stats
                                        .3
                                        .push((error_msg, file_path.to_string()));
                                    Self::save_failed_email(config, file_path);

                                    // 检测关键SMTP错误（基于原始错误字符串）
                                    if is_connection_error(&raw_error) {
                                        warn!("{}", tr_with_args("core.mailer.data_connection_error", &[("error", &raw_error)]));
                                        break;
                                    }
                                }
                                Err(_) => {
                                    error!("{}", tr_with_args("core.mailer.email_send_timeout_for", &[("path", file_path.as_str())]));
                                    group_stats
                                        .3
                                        .push((tr("core.mailer.error_send_timeout"), file_path.to_string()));
                                    Self::save_failed_email(config, file_path);
                                }
                            }
                        }
                    }
                }
            }

            // 添加RSET命令：如果还有更多邮件要发送，重置SMTP状态
            if email_idx + 1 < files.len() && running.load(Ordering::SeqCst) {
                info!("{}", tr_with_args("core.mailer.rset_command", &[("current", &(email_idx + 1).to_string()), ("total", &files.len().to_string())]));
                if let Err(e) = client.rset().await {
                    warn!("{}", tr_with_args("core.mailer.rset_command_failed", &[("current", &(email_idx + 1).to_string()), ("total", &files.len().to_string()), ("error", &e.to_string())]));
                    // RSET失败通常意味着连接有问题，提前退出批次
                    break;
                }
            }

            if config.email_send_interval_ms > 0
                && email_idx + 1 < files.len()
                && running.load(Ordering::SeqCst)
            {
                info!("{}", tr_with_args("core.mailer.waiting_next_email", &[("ms", &config.email_send_interval_ms.to_string()), ("current", &(email_idx + 1).to_string()), ("total", &files.len().to_string())]));
                let sleep_duration =
                    std::time::Duration::from_millis(config.email_send_interval_ms);
                let running_clone_for_sleep = running.clone();
                tokio::select! {
                    biased;
                    _ = async { loop { if !running_clone_for_sleep.load(Ordering::SeqCst) { break; } tokio::time::sleep(Duration::from_millis(100)).await; } } => {
                        warn!("{}", tr_with_args("core.mailer.email_interval_interrupted", &[("current", &(email_idx + 1).to_string()), ("total", &files.len().to_string())]));
                    }
                    _ = tokio::time::sleep(sleep_duration) => {}
                }
                if !running.load(Ordering::SeqCst) {
                    warn!("{}", tr_with_args("core.mailer.email_interval_exit", &[("current", &(email_idx + 1).to_string()), ("total", &files.len().to_string())]));
                    break;
                }
            }
        }
        Ok(())
    }
}
