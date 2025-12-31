use anyhow::Result;
use log::{Level, Log, Metadata, Record, SetLoggerError};
use rsendmail_core::{Config, Mailer, Stats};
use slint::{ComponentHandle, Model, ModelRc, SharedString, VecModel};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

mod i18n;

slint::include_modules!();

// 发送事件
enum SendEvent {
    Log { level: String, message: String },
    Progress { sent: i32, success: i32, fail: i32 },
    Stats { qps: f32, elapsed: String },
    RoundStart { current: i32, total: i32 },
    Completed { stats: Stats },
    Stopped,
    Error { message: String },
}

// 自定义 Logger，同时输出到终端和 GUI
struct GuiLogger {
    tx: Mutex<Option<tokio::sync::mpsc::Sender<SendEvent>>>,
}

impl GuiLogger {
    fn set_sender(&self, sender: tokio::sync::mpsc::Sender<SendEvent>) {
        if let Ok(mut tx) = self.tx.lock() {
            *tx = Some(sender);
        }
    }

    fn clear_sender(&self) {
        if let Ok(mut tx) = self.tx.lock() {
            *tx = None;
        }
    }
}

impl Log for GuiLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let level = match record.level() {
                Level::Error => "ERROR",
                Level::Warn => "WARN",
                Level::Info => "INFO",
                Level::Debug => "DEBUG",
                Level::Trace => "TRACE",
            };
            let message = format!("{}", record.args());

            // 输出到终端
            let time = chrono::Local::now().format("%H:%M:%S");
            eprintln!("{} [{}] {}", time, level, message);

            // 发送到 GUI（如果已设置）
            if let Ok(tx_guard) = self.tx.lock() {
                if let Some(tx) = tx_guard.as_ref() {
                    let _ = tx.try_send(SendEvent::Log {
                        level: level.to_string(),
                        message,
                    });
                }
            }
        }
    }

    fn flush(&self) {}
}

static GUI_LOGGER: GuiLogger = GuiLogger {
    tx: Mutex::new(None),
};

fn init_logger() -> Result<(), SetLoggerError> {
    log::set_logger(&GUI_LOGGER).map(|()| log::set_max_level(log::LevelFilter::Info))
}

fn main() -> Result<()> {
    // 初始化自定义日志
    init_logger().expect("Failed to initialize logger");

    // 创建 Slint 应用
    let app = AppWindow::new()?;

    // 初始化 i18n
    setup_i18n(&app);

    // 创建用于取消发送的标志
    let running = Arc::new(AtomicBool::new(false));

    // 设置回调
    setup_callbacks(&app, running.clone());

    // 运行应用
    app.run()?;

    Ok(())
}

fn setup_i18n(app: &AppWindow) {
    // 设置语言列表
    let languages: Vec<SharedString> = i18n::language_names()
        .into_iter()
        .map(|s| s.into())
        .collect();
    app.set_available_languages(ModelRc::new(VecModel::from(languages)));

    // 设置当前语言索引
    let current_lang = i18n::current_language();
    app.set_current_language_index(current_lang.index() as i32);

    // 更新 UI 文本
    update_ui_texts(app);

    // 语言切换回调
    let app_weak = app.as_weak();
    app.on_language_changed(move |index| {
        let lang = i18n::Language::from_index(index as usize);
        i18n::set_language(lang);

        if let Some(app) = app_weak.upgrade() {
            update_ui_texts(&app);
        }
    });
}

fn update_ui_texts(app: &AppWindow) {
    // 更新所有 UI 文本
    app.set_tr_smtp_server(i18n::t("smtp-server").into());
    app.set_tr_server_address(i18n::t("server-address").into());
    app.set_tr_port(i18n::t("port").into());
    app.set_tr_use_tls(i18n::t("use-tls").into());
    app.set_tr_accept_invalid_certs(i18n::t("accept-invalid-certs").into());
    app.set_tr_auth_required(i18n::t("auth-required").into());
    app.set_tr_username(i18n::t("username").into());
    app.set_tr_password(i18n::t("password").into());
    app.set_tr_sender(i18n::t("sender").into());
    app.set_tr_recipient(i18n::t("recipient").into());
    app.set_tr_recipient_hint(i18n::t("recipient-hint").into());

    app.set_tr_send_mode(i18n::t("send-mode").into());
    app.set_tr_eml_batch(i18n::t("eml-batch").into());
    app.set_tr_single_attachment(i18n::t("single-attachment").into());
    app.set_tr_dir_attachment(i18n::t("dir-attachment").into());
    app.set_tr_eml_directory(i18n::t("eml-directory").into());
    app.set_tr_attachment_file(i18n::t("attachment-file").into());
    app.set_tr_attachment_directory(i18n::t("attachment-directory").into());
    app.set_tr_extension(i18n::t("extension").into());
    app.set_tr_browse(i18n::t("browse").into());
    app.set_tr_email_subject(i18n::t("email-subject").into());
    app.set_tr_email_body(i18n::t("email-body").into());
    app.set_tr_filename_hint(i18n::t("filename-hint").into());

    app.set_tr_advanced_options(i18n::t("advanced-options").into());
    app.set_tr_performance(i18n::t("performance").into());
    app.set_tr_processes(i18n::t("processes").into());
    app.set_tr_batch_size(i18n::t("batch-size").into());
    app.set_tr_send_interval(i18n::t("send-interval").into());
    app.set_tr_timeout(i18n::t("timeout").into());
    app.set_tr_loop_settings(i18n::t("loop-settings").into());
    app.set_tr_infinite_loop(i18n::t("infinite-loop").into());
    app.set_tr_repeat_count(i18n::t("repeat-count").into());
    app.set_tr_loop_interval(i18n::t("loop-interval").into());
    app.set_tr_retry_interval(i18n::t("retry-interval").into());
    app.set_tr_email_processing(i18n::t("email-processing").into());
    app.set_tr_keep_headers(i18n::t("keep-headers").into());
    app.set_tr_modify_headers(i18n::t("modify-headers").into());
    app.set_tr_anonymize_emails(i18n::t("anonymize-emails").into());
    app.set_tr_domain(i18n::t("domain").into());
    app.set_tr_logging(i18n::t("logging").into());
    app.set_tr_log_level(i18n::t("log-level").into());
    app.set_tr_log_file(i18n::t("log-file").into());
    app.set_tr_failed_emails_dir(i18n::t("failed-emails-dir").into());
    app.set_tr_optional(i18n::t("optional").into());

    app.set_tr_statistics(i18n::t("statistics").into());
    app.set_tr_total(i18n::t("total").into());
    app.set_tr_success(i18n::t("success").into());
    app.set_tr_failed(i18n::t("failed").into());
    app.set_tr_current_round(i18n::t("current-round").into());
    app.set_tr_elapsed_time(i18n::t("elapsed-time").into());

    app.set_tr_send_log(i18n::t("send-log").into());
    app.set_tr_clear(i18n::t("clear").into());
    app.set_tr_export_log(i18n::t("export-log").into());

    app.set_tr_save_config(i18n::t("save-config").into());
    app.set_tr_load_config(i18n::t("load-config").into());
    app.set_tr_test_connection(i18n::t("test-connection").into());
    app.set_tr_start_send(i18n::t("start-send").into());
    app.set_tr_stop_send(i18n::t("stop-send").into());

    app.set_tr_language(i18n::t("language").into());
    app.set_tr_theme(i18n::t("theme").into());
    app.set_tr_ok(i18n::t("ok").into());

    // 更新状态文本
    update_status_text(app);
}

fn update_status_text(app: &AppWindow) {
    let status = app.get_status();
    let text = match status {
        SendStatus::Idle => i18n::t("status-ready"),
        SendStatus::Preparing => i18n::t("status-preparing"),
        SendStatus::Sending => i18n::t("status-sending"),
        SendStatus::Stopped => i18n::t("status-stopped"),
        SendStatus::Completed => i18n::t("status-completed"),
    };
    app.set_status_text(text.into());
}

fn setup_callbacks(app: &AppWindow, running: Arc<AtomicBool>) {
    let app_weak = app.as_weak();

    // 关闭消息对话框
    {
        let app_weak = app_weak.clone();
        app.on_close_message_dialog(move || {
            let app = app_weak.unwrap();
            app.set_show_message_dialog(false);
        });
    }

    // 测试连接
    {
        let app_weak = app_weak.clone();
        app.on_test_connection(move || {
            let app = app_weak.unwrap();
            add_log(&app, "INFO", &i18n::t("status-preparing"));

            let config = build_config_from_ui(&app);

            // 验证必填字段
            if config.smtp_server.is_empty() {
                show_error(&app, &i18n::t("error-no-smtp-server"));
                return;
            }
            if config.from.is_empty() {
                show_error(&app, &i18n::t("error-no-sender"));
                return;
            }

            add_log(
                &app,
                "INFO",
                &format!(
                    "连接到 {}:{} (TLS: {})",
                    config.smtp_server, config.port, config.use_tls
                ),
            );

            // TODO: 实际测试连接逻辑
            add_log(&app, "INFO", "连接测试功能待实现");
        });
    }

    // 开始发送
    {
        let app_weak = app_weak.clone();
        let running = running.clone();
        app.on_start_send(move || {
            let app = app_weak.unwrap();
            let config = build_config_from_ui(&app);

            // 验证配置
            if let Err(msg) = validate_config(&config, &app) {
                show_error(&app, &msg);
                add_log(&app, "ERROR", &msg);
                return;
            }

            // 更新状态
            app.set_status(SendStatus::Preparing);
            app.set_status_text("准备中...".into());
            app.set_sent_count(0);
            app.set_success_count(0);
            app.set_fail_count(0);

            // 设置 running 标志
            running.store(true, Ordering::SeqCst);

            // 创建通道
            let (tx, mut rx) = mpsc::channel::<SendEvent>(100);

            // 设置 logger sender，使 log crate 的日志也能发送到 GUI
            GUI_LOGGER.set_sender(tx.clone());

            // 在后台线程运行发送任务
            let config_clone = config.clone();
            let running_clone = running.clone();
            let tx_clone = tx.clone();

            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async move {
                    run_send_task(config_clone, running_clone, tx_clone).await;
                });
                // 任务结束后清除 sender
                GUI_LOGGER.clear_sender();
            });

            // 在主线程处理事件
            let app_weak_for_events = app_weak.clone();
            let running_for_events = running.clone();
            slint::spawn_local(async move {
                while let Some(event) = rx.recv().await {
                    if let Some(app) = app_weak_for_events.upgrade() {
                        match event {
                            SendEvent::Log { level, message } => {
                                add_log(&app, &level, &message);
                            }
                            SendEvent::Progress {
                                sent,
                                success,
                                fail,
                            } => {
                                app.set_sent_count(sent);
                                app.set_success_count(success);
                                app.set_fail_count(fail);
                            }
                            SendEvent::Stats { qps, elapsed } => {
                                app.set_qps(qps);
                                app.set_elapsed_time(elapsed.into());
                            }
                            SendEvent::RoundStart { current, total } => {
                                app.set_current_round(current);
                                app.set_total_rounds(total);
                                app.set_status(SendStatus::Sending);
                                app.set_status_text("发送中...".into());
                            }
                            SendEvent::Completed { stats } => {
                                app.set_status(SendStatus::Completed);
                                app.set_status_text("完成".into());
                                app.set_total_count(stats.email_count as i32);
                                running_for_events.store(false, Ordering::SeqCst);
                                add_log(
                                    &app,
                                    "INFO",
                                    &format!(
                                        "发送完成！成功: {}, 失败: {}",
                                        stats.email_count - stats.send_errors - stats.parse_errors,
                                        stats.send_errors + stats.parse_errors
                                    ),
                                );
                            }
                            SendEvent::Stopped => {
                                app.set_status(SendStatus::Stopped);
                                app.set_status_text("已停止".into());
                                running_for_events.store(false, Ordering::SeqCst);
                            }
                            SendEvent::Error { message } => {
                                add_log(&app, "ERROR", &message);
                                app.set_status(SendStatus::Stopped);
                                app.set_status_text("错误".into());
                                running_for_events.store(false, Ordering::SeqCst);
                            }
                        }
                    }
                }
            })
            .unwrap();
        });
    }

    // 停止发送
    {
        let app_weak = app_weak.clone();
        let running = running.clone();
        app.on_stop_send(move || {
            let app = app_weak.unwrap();
            add_log(&app, "WARN", "正在停止发送...");
            running.store(false, Ordering::SeqCst);
            app.set_status_text("停止中...".into());
        });
    }

    // 浏览 EML 目录
    {
        let app_weak = app_weak.clone();
        app.on_browse_eml_dir(move || {
            let app = app_weak.unwrap();
            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                let path_str = path.to_string_lossy().to_string();
                app.set_eml_dir(path_str.clone().into());

                // 扫描文件数量
                let extension = app.get_eml_extension().to_string();
                let count = count_files_with_extension(&path_str, &extension);
                app.set_eml_file_count(count);
                app.set_total_count(count);
            }
        });
    }

    // 浏览单个附件
    {
        let app_weak = app_weak.clone();
        app.on_browse_attachment(move || {
            let app = app_weak.unwrap();
            if let Some(path) = rfd::FileDialog::new().pick_file() {
                app.set_attachment_path(path.to_string_lossy().to_string().into());
                app.set_total_count(1);
            }
        });
    }

    // 浏览附件目录
    {
        let app_weak = app_weak.clone();
        app.on_browse_attachment_dir(move || {
            let app = app_weak.unwrap();
            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                let path_str = path.to_string_lossy().to_string();
                app.set_attachment_dir(path_str.clone().into());

                // 扫描文件数量
                let count = count_all_files(&path_str);
                app.set_attachment_file_count(count);
                app.set_total_count(count);
            }
        });
    }

    // 浏览日志文件
    {
        let app_weak = app_weak.clone();
        app.on_browse_log_file(move || {
            let app = app_weak.unwrap();
            if let Some(path) = rfd::FileDialog::new().save_file() {
                app.set_log_file(path.to_string_lossy().to_string().into());
            }
        });
    }

    // 浏览失败邮件目录
    {
        let app_weak = app_weak.clone();
        app.on_browse_failed_dir(move || {
            let app = app_weak.unwrap();
            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                app.set_failed_emails_dir(path.to_string_lossy().to_string().into());
            }
        });
    }

    // 清空日志
    {
        let app_weak = app_weak.clone();
        app.on_clear_logs(move || {
            let app = app_weak.unwrap();
            app.set_logs(ModelRc::new(VecModel::from(vec![])));
        });
    }

    // 导出日志
    {
        let app_weak = app_weak.clone();
        app.on_export_logs(move || {
            let app = app_weak.unwrap();
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Log files", &["log", "txt"])
                .save_file()
            {
                let logs = app.get_logs();
                let mut content = String::new();
                for i in 0..logs.row_count() {
                    if let Some(entry) = logs.row_data(i) {
                        content.push_str(&format!(
                            "[{}] [{}] {}\n",
                            entry.timestamp, entry.level, entry.message
                        ));
                    }
                }
                if let Err(e) = std::fs::write(&path, content) {
                    add_log(&app, "ERROR", &format!("导出日志失败: {}", e));
                } else {
                    add_log(&app, "INFO", &format!("日志已导出到: {}", path.display()));
                }
            }
        });
    }

    // 保存配置
    {
        let app_weak = app_weak.clone();
        app.on_save_config(move || {
            let app = app_weak.unwrap();
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("JSON files", &["json"])
                .save_file()
            {
                let config = build_config_from_ui(&app);
                match serde_json::to_string_pretty(&config) {
                    Ok(json) => {
                        if let Err(e) = std::fs::write(&path, json) {
                            add_log(&app, "ERROR", &format!("保存配置失败: {}", e));
                        } else {
                            add_log(&app, "INFO", &format!("配置已保存到: {}", path.display()));
                        }
                    }
                    Err(e) => {
                        add_log(&app, "ERROR", &format!("序列化配置失败: {}", e));
                    }
                }
            }
        });
    }

    // 加载配置
    {
        let app_weak = app_weak.clone();
        app.on_load_config(move || {
            let app = app_weak.unwrap();
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("JSON files", &["json"])
                .pick_file()
            {
                match std::fs::read_to_string(&path) {
                    Ok(content) => match serde_json::from_str::<Config>(&content) {
                        Ok(config) => {
                            apply_config_to_ui(&app, &config);
                            add_log(&app, "INFO", &format!("配置已加载: {}", path.display()));
                        }
                        Err(e) => {
                            add_log(&app, "ERROR", &format!("解析配置失败: {}", e));
                        }
                    },
                    Err(e) => {
                        add_log(&app, "ERROR", &format!("读取配置失败: {}", e));
                    }
                }
            }
        });
    }
}

fn add_log(app: &AppWindow, level: &str, message: &str) {
    let timestamp = chrono::Local::now().format("%H:%M:%S").to_string();

    // 清理消息中的多余空白字符：将制表符和多个连续空格替换为单个空格
    let cleaned_message: String = message
        .replace('\t', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    let entry = LogEntry {
        timestamp: timestamp.into(),
        level: level.into(),
        message: cleaned_message.into(),
    };

    let logs = app.get_logs();
    let mut new_logs: Vec<LogEntry> = (0..logs.row_count())
        .filter_map(|i| logs.row_data(i))
        .collect();
    new_logs.push(entry);

    // 限制日志数量
    if new_logs.len() > 1000 {
        new_logs = new_logs.split_off(new_logs.len() - 1000);
    }

    app.set_logs(ModelRc::new(VecModel::from(new_logs)));
}

fn show_error(app: &AppWindow, message: &str) {
    app.set_message_dialog_title(i18n::t("error-title").into());
    app.set_message_dialog_content(message.into());
    app.set_message_dialog_is_error(true);
    app.set_show_message_dialog(true);
}

#[allow(dead_code)]
fn show_message(app: &AppWindow, title: &str, message: &str) {
    app.set_message_dialog_title(title.into());
    app.set_message_dialog_content(message.into());
    app.set_message_dialog_is_error(false);
    app.set_show_message_dialog(true);
}

fn parse_u16(s: &str, default: u16) -> u16 {
    s.parse().unwrap_or(default)
}

fn parse_u32(s: &str, default: u32) -> u32 {
    s.parse().unwrap_or(default)
}

fn parse_u64(s: &str, default: u64) -> u64 {
    s.parse().unwrap_or(default)
}

fn parse_usize(s: &str, default: usize) -> usize {
    s.parse().unwrap_or(default)
}

fn build_config_from_ui(app: &AppWindow) -> Config {
    let send_mode = app.get_send_mode();

    let (dir, attachment, attachment_dir) = match send_mode {
        SendMode::EmlBatch => {
            let dir = app.get_eml_dir().to_string();
            (if dir.is_empty() { None } else { Some(dir) }, None, None)
        }
        SendMode::SingleAttachment => {
            let path = app.get_attachment_path().to_string();
            (None, if path.is_empty() { None } else { Some(path) }, None)
        }
        SendMode::DirAttachment => {
            let dir = app.get_attachment_dir().to_string();
            (None, None, if dir.is_empty() { None } else { Some(dir) })
        }
    };

    let subject = app.get_subject_template().to_string();
    let text = app.get_text_template().to_string();
    let log_file = app.get_log_file().to_string();
    let failed_dir = app.get_failed_emails_dir().to_string();

    Config {
        smtp_server: app.get_smtp_server().to_string(),
        port: parse_u16(app.get_smtp_port_str().as_ref(), 25),
        from: app.get_from_address().to_string(),
        to: app.get_to_address().to_string(),
        dir,
        extension: app.get_eml_extension().to_string(),
        processes: app.get_processes().to_string(),
        batch_size: parse_usize(app.get_batch_size_str().as_ref(), 1),
        smtp_timeout: parse_u64(app.get_smtp_timeout_str().as_ref(), 30),
        log_level: app.get_log_level().to_string(),
        keep_headers: app.get_keep_headers(),
        anonymize_emails: app.get_anonymize_emails(),
        anonymize_domain: app.get_anonymize_domain().to_string(),
        modify_headers: app.get_modify_headers(),
        r#loop: app.get_loop_mode(),
        repeat: parse_u32(app.get_repeat_count_str().as_ref(), 1),
        loop_interval: parse_u64(app.get_loop_interval_str().as_ref(), 1),
        retry_interval: parse_u64(app.get_retry_interval_str().as_ref(), 5),
        attachment,
        attachment_dir,
        subject_template: if subject.is_empty() {
            None
        } else {
            Some(subject)
        },
        text_template: if text.is_empty() { None } else { Some(text) },
        html_template: None,
        email_send_interval_ms: parse_u64(app.get_email_interval_str().as_ref(), 0),
        auth_mode: app.get_auth_mode(),
        username: if app.get_auth_mode() {
            Some(app.get_username().to_string())
        } else {
            None
        },
        password: if app.get_auth_mode() {
            Some(app.get_password().to_string())
        } else {
            None
        },
        use_tls: app.get_use_tls(),
        accept_invalid_certs: app.get_accept_invalid_certs(),
        failed_emails_dir: if failed_dir.is_empty() {
            None
        } else {
            Some(failed_dir)
        },
        log_file: if log_file.is_empty() {
            None
        } else {
            Some(log_file)
        },
    }
}

fn apply_config_to_ui(app: &AppWindow, config: &Config) {
    app.set_smtp_server(config.smtp_server.clone().into());
    app.set_smtp_port_str(config.port.to_string().into());
    app.set_from_address(config.from.clone().into());
    app.set_to_address(config.to.clone().into());
    app.set_use_tls(config.use_tls);
    app.set_accept_invalid_certs(config.accept_invalid_certs);
    app.set_auth_mode(config.auth_mode);
    if let Some(ref username) = config.username {
        app.set_username(username.clone().into());
    }
    // 不加载密码

    // 设置发送模式
    if config.dir.is_some() {
        app.set_send_mode(SendMode::EmlBatch);
        if let Some(ref dir) = config.dir {
            app.set_eml_dir(dir.clone().into());
        }
    } else if config.attachment.is_some() {
        app.set_send_mode(SendMode::SingleAttachment);
        if let Some(ref path) = config.attachment {
            app.set_attachment_path(path.clone().into());
        }
    } else if config.attachment_dir.is_some() {
        app.set_send_mode(SendMode::DirAttachment);
        if let Some(ref dir) = config.attachment_dir {
            app.set_attachment_dir(dir.clone().into());
        }
    }

    app.set_eml_extension(config.extension.clone().into());
    app.set_processes(config.processes.clone().into());
    app.set_batch_size_str(config.batch_size.to_string().into());
    app.set_smtp_timeout_str(config.smtp_timeout.to_string().into());
    app.set_email_interval_str(config.email_send_interval_ms.to_string().into());
    app.set_loop_mode(config.r#loop);
    app.set_repeat_count_str(config.repeat.to_string().into());
    app.set_loop_interval_str(config.loop_interval.to_string().into());
    app.set_retry_interval_str(config.retry_interval.to_string().into());
    app.set_keep_headers(config.keep_headers);
    app.set_modify_headers(config.modify_headers);
    app.set_anonymize_emails(config.anonymize_emails);
    app.set_anonymize_domain(config.anonymize_domain.clone().into());
    app.set_log_level(config.log_level.clone().into());

    if let Some(ref template) = config.subject_template {
        app.set_subject_template(template.clone().into());
    }
    if let Some(ref template) = config.text_template {
        app.set_text_template(template.clone().into());
    }
    if let Some(ref path) = config.log_file {
        app.set_log_file(path.clone().into());
    }
    if let Some(ref dir) = config.failed_emails_dir {
        app.set_failed_emails_dir(dir.clone().into());
    }
}

fn validate_config(config: &Config, app: &AppWindow) -> Result<(), String> {
    if config.smtp_server.is_empty() {
        return Err(i18n::t("error-no-smtp-server"));
    }
    if config.from.is_empty() {
        return Err(i18n::t("error-no-sender"));
    }
    if config.to.is_empty() {
        return Err(i18n::t("error-no-recipient"));
    }

    let send_mode = app.get_send_mode();
    match send_mode {
        SendMode::EmlBatch => {
            if config.dir.is_none() {
                return Err(i18n::t("error-no-eml-dir"));
            }
        }
        SendMode::SingleAttachment => {
            if config.attachment.is_none() {
                return Err(i18n::t("error-no-attachment"));
            }
        }
        SendMode::DirAttachment => {
            if config.attachment_dir.is_none() {
                return Err(i18n::t("error-no-attachment-dir"));
            }
        }
    }

    if config.auth_mode {
        if config.username.as_ref().is_none_or(|s| s.is_empty()) {
            return Err(i18n::t("error-no-username"));
        }
        if config.password.as_ref().is_none_or(|s| s.is_empty()) {
            return Err(i18n::t("error-no-password"));
        }
    }

    Ok(())
}

fn count_files_with_extension(dir: &str, extension: &str) -> i32 {
    walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().is_file()
                && e.path()
                    .extension()
                    .is_some_and(|ext| ext.to_string_lossy() == extension)
        })
        .count() as i32
}

fn count_all_files(dir: &str) -> i32 {
    walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .count() as i32
}

async fn run_send_task(config: Config, running: Arc<AtomicBool>, tx: mpsc::Sender<SendEvent>) {
    let mailer = Mailer::new(config.clone());

    let total_rounds = if config.r#loop {
        i32::MAX
    } else {
        config.repeat as i32
    };

    let mut current_round = 1;
    let start_time = Instant::now();

    while current_round <= total_rounds && running.load(Ordering::SeqCst) {
        let _ = tx
            .send(SendEvent::RoundStart {
                current: current_round,
                total: if config.r#loop { -1 } else { total_rounds },
            })
            .await;

        let _ = tx
            .send(SendEvent::Log {
                level: "INFO".to_string(),
                message: format!(
                    "开始第 {}/{} 轮发送",
                    current_round,
                    if config.r#loop {
                        "∞".to_string()
                    } else {
                        total_rounds.to_string()
                    }
                ),
            })
            .await;

        match mailer.send_all_with_cancel(running.clone()).await {
            Ok(stats) => {
                let elapsed = start_time.elapsed();
                let elapsed_str = format!(
                    "{:02}:{:02}:{:02}",
                    elapsed.as_secs() / 3600,
                    (elapsed.as_secs() % 3600) / 60,
                    elapsed.as_secs() % 60
                );

                let total_errors = stats.send_errors + stats.parse_errors;
                let success = if stats.email_count > total_errors {
                    stats.email_count - total_errors
                } else {
                    0
                };
                let fail = total_errors;
                let qps = if elapsed.as_secs_f32() > 0.0 {
                    stats.email_count as f32 / elapsed.as_secs_f32()
                } else {
                    0.0
                };

                let _ = tx
                    .send(SendEvent::Progress {
                        sent: stats.email_count as i32,
                        success: success as i32,
                        fail: fail as i32,
                    })
                    .await;

                let _ = tx
                    .send(SendEvent::Stats {
                        qps,
                        elapsed: elapsed_str,
                    })
                    .await;

                let _ = tx
                    .send(SendEvent::Log {
                        level: "INFO".to_string(),
                        message: format!("第 {} 轮发送完成", current_round),
                    })
                    .await;

                // 检查是否需要继续
                if current_round < total_rounds && running.load(Ordering::SeqCst) {
                    let _ = tx
                        .send(SendEvent::Log {
                            level: "INFO".to_string(),
                            message: format!("等待 {} 秒后开始下一轮...", config.loop_interval),
                        })
                        .await;
                    tokio::time::sleep(Duration::from_secs(config.loop_interval)).await;
                }

                // 最后一轮完成
                if current_round >= total_rounds || !running.load(Ordering::SeqCst) {
                    let _ = tx.send(SendEvent::Completed { stats }).await;
                    break;
                }
            }
            Err(e) => {
                let _ = tx
                    .send(SendEvent::Error {
                        message: format!("发送失败: {}", e),
                    })
                    .await;
                break;
            }
        }

        current_round += 1;
    }

    if !running.load(Ordering::SeqCst) {
        let _ = tx.send(SendEvent::Stopped).await;
    }
}
