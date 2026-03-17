use log::LevelFilter;
use rsendmail_i18n::tr_with_args;
use simplelog::*;
use std::fs::OpenOptions;

pub fn init_logging(level: LevelFilter, log_file: Option<&str>) {
    // 配置日志格式
    let mut config_builder = ConfigBuilder::new();
    config_builder.set_time_format_rfc3339();
    let _ = config_builder.set_time_offset_to_local();
    let log_config = config_builder.build();

    if let Some(log_file_path) = log_file {
        // 如果指定了日志文件，同时输出到控制台和文件（追加模式）
        let log_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file_path)
            .unwrap_or_else(|e| {
                panic!(
                    "{}",
                    tr_with_args(
                        "cli_logging.create_log_file_failed",
                        &[("path", log_file_path), ("error", &e.to_string())]
                    )
                )
            });

        CombinedLogger::init(vec![
            TermLogger::new(
                level,
                log_config.clone(),
                TerminalMode::Mixed,
                ColorChoice::Auto,
            ),
            WriteLogger::new(level, log_config, log_file),
        ])
        .unwrap_or_else(|e| panic!("{}", tr_with_args("cli_logging.init_log_failed", &[("error", &e.to_string())])));

        log::info!(
            "{}",
            tr_with_args("cli_logging.log_to_console_and_file", &[("path", log_file_path)])
        );
    } else {
        // 如果没有指定日志文件，只输出到控制台
        TermLogger::init(level, log_config, TerminalMode::Mixed, ColorChoice::Auto)
            .unwrap_or_else(|e| panic!("{}", tr_with_args("cli_logging.init_log_failed", &[("error", &e.to_string())])));
    }
}
