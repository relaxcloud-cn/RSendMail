use log::{error, info, warn};
use rsendmail_i18n::{set_language, tr, tr_with_args};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

mod args;
mod logging;

use args::{detect_language, parse_args};
use rsendmail_core::{Mailer, Stats};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Detect and set language BEFORE parsing CLI args
    // This ensures --help shows localized text
    let lang = detect_language();
    set_language(lang);

    // Parse CLI args with localized help
    let config = parse_args();

    // Initialize logging
    let log_level = config.get_log_level();
    logging::init_logging(log_level, config.log_file.as_deref());

    // Create atomic bool for graceful shutdown
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    // Setup Ctrl+C handler
    ctrlc::set_handler(move || {
        warn!("{}", tr("cli_main.interrupted"));
        r.store(false, Ordering::SeqCst);
    })?;

    // Create mailer
    let mailer = Mailer::new(config.clone());

    // Set iteration count
    let mut iteration_count = if config.r#loop {
        u32::MAX
    } else {
        config.repeat
    };

    // Track overall statistics
    let mut total_stats = Stats::new();
    let total_start_time = Instant::now();
    let mut successful_iterations = 0;

    // Main send loop
    let mut current_iteration = 1;
    while iteration_count > 0 && running.load(Ordering::SeqCst) {
        let total_str = if config.r#loop {
            "∞".to_string()
        } else {
            config.repeat.to_string()
        };
        info!(
            "{}",
            tr_with_args(
                "cli_main.starting_round",
                &[
                    ("current", &current_iteration.to_string()),
                    ("total", &total_str)
                ]
            )
        );

        // Send emails and get stats
        match mailer.send_all_with_cancel(running.clone()).await {
            Ok(stats) => {
                successful_iterations += 1;

                // Accumulate stats
                total_stats.email_count += stats.email_count;

                for duration in &stats.parse_durations {
                    total_stats.parse_durations.push(*duration);
                }
                for duration in &stats.send_durations {
                    total_stats.send_durations.push(*duration);
                }

                total_stats.parse_errors += stats.parse_errors;
                total_stats.send_errors += stats.send_errors;

                // Accumulate error details
                for (error_type, count) in &stats.error_details {
                    let entry = total_stats
                        .error_details
                        .entry(error_type.clone())
                        .or_insert(0);
                    *entry += count;
                }

                // Accumulate failed files
                for (error_type, files) in &stats.failed_files {
                    total_stats
                        .failed_files
                        .entry(error_type.clone())
                        .or_default()
                        .extend(files.clone());
                }

                info!(
                    "{}",
                    tr_with_args(
                        "cli_main.round_completed",
                        &[("round", &current_iteration.to_string())]
                    )
                );
                info!("{}", stats);

                // Wait before next iteration if not the last one
                if iteration_count > 1 && running.load(Ordering::SeqCst) {
                    info!(
                        "{}",
                        tr_with_args(
                            "cli_main.waiting_next_round",
                            &[("seconds", &config.loop_interval.to_string())]
                        )
                    );
                    tokio::time::sleep(Duration::from_secs(config.loop_interval)).await;
                }
            }
            Err(e) => {
                error!(
                    "{}",
                    tr_with_args(
                        "cli_main.round_failed",
                        &[
                            ("round", &current_iteration.to_string()),
                            ("error", &e.to_string())
                        ]
                    )
                );
                // Continue if in loop mode and not interrupted
                if !config.r#loop || !running.load(Ordering::SeqCst) {
                    return Err(e);
                }
                // Wait and retry
                info!(
                    "{}",
                    tr_with_args(
                        "cli_main.waiting_next_round",
                        &[("seconds", &config.retry_interval.to_string())]
                    )
                );
                tokio::time::sleep(Duration::from_secs(config.retry_interval)).await;
            }
        }

        current_iteration += 1;
        iteration_count -= 1;
    }

    // Show overall stats
    if successful_iterations > 0 {
        total_stats.total_duration = total_start_time.elapsed();
        info!(
            "{}",
            tr_with_args(
                "cli_main.all_rounds_completed",
                &[("count", &successful_iterations.to_string())]
            )
        );
        info!("{}", total_stats);
    }

    Ok(())
}
