use log::{Level, Log, Metadata, Record, SetLoggerError};
use rsendmail_core::{Config, Mailer};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tauri::{AppHandle, Emitter, State};
use tokio::time::Duration;

pub struct AppState {
    pub is_running: Arc<AtomicBool>,
}

// Global logger that forwards `log::info!` directly to the Vue frontend
struct TauriLogger {
    app_handle: Mutex<Option<AppHandle>>,
}

impl TauriLogger {
    fn set_handle(&self, handle: AppHandle) {
        if let Ok(mut lock) = self.app_handle.lock() {
            *lock = Some(handle);
        }
    }
}

impl Log for TauriLogger {
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

            // Print native standard output
            let time = chrono::Local::now().format("%H:%M:%S");
            eprintln!("{} [{}] {}", time, level, message);

            // Forward asynchronously to the Vue GUI if it's connected
            if let Ok(lock) = self.app_handle.lock() {
                if let Some(app) = lock.as_ref() {
                    let _ = app.emit(
                        "send-event",
                        LogPayload {
                            level: level.to_string(),
                            message,
                        },
                    );
                }
            }
        }
    }

    fn flush(&self) {}
}

static TAURI_LOGGER: TauriLogger = TauriLogger {
    app_handle: Mutex::new(None),
};

fn init_logger() -> Result<(), SetLoggerError> {
    log::set_logger(&TAURI_LOGGER).map(|()| log::set_max_level(log::LevelFilter::Info))
}

#[derive(Clone, serde::Serialize)]
struct LogPayload {
    level: String,
    message: String,
}

#[derive(Clone, serde::Serialize)]
struct ProgressPayload {
    sent: u32,
    success: u32,
    fail: u32,
}

#[derive(Clone, serde::Serialize)]
struct StatsPayload {
    qps: f32,
    elapsed: String,
}

#[tauri::command]
async fn start_sending(
    config: Config,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<(), String> {
    if state.is_running.load(Ordering::SeqCst) {
        return Err("Sending is already running.".to_string());
    }

    state.is_running.store(true, Ordering::SeqCst);
    let running_clone = state.is_running.clone();

    let total_rounds = if config.r#loop {
        i32::MAX
    } else {
        config.repeat as i32
    };

    // Spawn task so we immediately return Ok and unblock UI.
    tokio::spawn(async move {
        let mailer = Mailer::new(config.clone());
        let mut current_round = 1;
        let start_time = Instant::now();

        while current_round <= total_rounds && running_clone.load(Ordering::SeqCst) {
            let _ = app.emit(
                "send-event",
                LogPayload {
                    level: "INFO".to_string(),
                    message: format!(
                        "Started round {}/{}",
                        current_round,
                        if config.r#loop {
                            "∞".into()
                        } else {
                            total_rounds.to_string()
                        }
                    ),
                },
            );

            match mailer.send_all_with_cancel(running_clone.clone()).await {
                Ok(stats) => {
                    let elapsed = start_time.elapsed();
                    let elapsed_str = format!(
                        "{:02}:{:02}:{:02}",
                        elapsed.as_secs() / 3600,
                        (elapsed.as_secs() % 3600) / 60,
                        elapsed.as_secs() % 60
                    );

                    let total_errors = stats.send_errors + stats.parse_errors;
                    let total_processed = stats.email_count + total_errors;

                    let qps = if elapsed.as_secs_f32() > 0.0 {
                        stats.email_count as f32 / elapsed.as_secs_f32()
                    } else {
                        0.0
                    };

                    let _ = app.emit(
                        "progress-event",
                        ProgressPayload {
                            sent: total_processed as u32,
                            success: stats.email_count as u32,
                            fail: total_errors as u32,
                        },
                    );

                    let _ = app.emit(
                        "stats-event",
                        StatsPayload {
                            qps,
                            elapsed: elapsed_str,
                        },
                    );

                    if current_round < total_rounds && running_clone.load(Ordering::SeqCst) {
                        tokio::time::sleep(Duration::from_secs(config.loop_interval)).await;
                    }

                    if current_round >= total_rounds || !running_clone.load(Ordering::SeqCst) {
                        let _ = app.emit(
                            "send-event",
                            LogPayload {
                                level: "SUCCESS".into(),
                                message: "All rounds completed.".into(),
                            },
                        );
                        break;
                    }
                }
                Err(e) => {
                    let _ = app.emit(
                        "send-event",
                        LogPayload {
                            level: "ERROR".into(),
                            message: format!("Send failed: {}", e),
                        },
                    );
                    break;
                }
            }
            current_round += 1;
        }

        running_clone.store(false, Ordering::SeqCst);
        let _ = app.emit(
            "send-event",
            LogPayload {
                level: "WARN".into(),
                message: "Engine stopped.".into(),
            },
        );
    });

    Ok(())
}

#[tauri::command]
fn stop_sending(state: State<'_, AppState>) -> Result<(), String> {
    state.is_running.store(false, Ordering::SeqCst);
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Mount the logger interceptor globally so `rsendmail_core` uses it
    let _ = init_logger();

    tauri::Builder::default()
        .setup(|app| {
            // Give the logger our Tauri window handle directly after boot
            TAURI_LOGGER.set_handle(app.handle().clone());
            Ok(())
        })
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState {
            is_running: Arc::new(AtomicBool::new(false)),
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![start_sending, stop_sending])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
