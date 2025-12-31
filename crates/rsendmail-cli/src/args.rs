//! CLI argument parsing with internationalization support
//!
//! This module uses clap's builder API instead of derive macros
//! to enable runtime i18n for help text.

use clap::{Arg, ArgAction, ArgMatches, Command};
use rsendmail_core::Config;
use rsendmail_i18n::{tr, Language};

/// Build the CLI command with localized help text
pub fn build_cli() -> Command {
    Command::new("rsendmail")
        .version(env!("CARGO_PKG_VERSION"))
        .author("RSendMail Contributors")
        .about(tr("cli.about"))
        // Required arguments
        .arg(
            Arg::new("smtp_server")
                .long("smtp-server")
                .help(tr("cli.smtp_server"))
                .required(true),
        )
        .arg(
            Arg::new("from")
                .long("from")
                .help(tr("cli.from"))
                .required(true),
        )
        .arg(
            Arg::new("to")
                .long("to")
                .help(tr("cli.to"))
                .required(true),
        )
        // Optional arguments with defaults
        .arg(
            Arg::new("port")
                .long("port")
                .help(tr("cli.port"))
                .default_value("25"),
        )
        .arg(
            Arg::new("dir")
                .long("dir")
                .help(tr("cli.dir"))
                .required_unless_present_any(["attachment", "attachment_dir"]),
        )
        .arg(
            Arg::new("extension")
                .long("extension")
                .help(tr("cli.extension"))
                .default_value("eml"),
        )
        .arg(
            Arg::new("processes")
                .long("processes")
                .help(tr("cli.processes"))
                .default_value("auto"),
        )
        .arg(
            Arg::new("batch_size")
                .long("batch-size")
                .help(tr("cli.batch_size"))
                .default_value("1"),
        )
        .arg(
            Arg::new("smtp_timeout")
                .long("smtp-timeout")
                .help(tr("cli.smtp_timeout"))
                .default_value("30"),
        )
        .arg(
            Arg::new("log_level")
                .long("log-level")
                .help(tr("cli.log_level"))
                .default_value("info"),
        )
        // Boolean flags
        .arg(
            Arg::new("keep_headers")
                .long("keep-headers")
                .help(tr("cli.keep_headers"))
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("anonymize_emails")
                .long("anonymize-emails")
                .help(tr("cli.anonymize_emails"))
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("anonymize_domain")
                .long("anonymize-domain")
                .help(tr("cli.anonymize_domain"))
                .default_value("example.com"),
        )
        .arg(
            Arg::new("modify_headers")
                .long("modify-headers")
                .help(tr("cli.modify_headers"))
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("loop")
                .long("loop")
                .help(tr("cli.loop"))
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("repeat")
                .long("repeat")
                .help(tr("cli.repeat"))
                .default_value("1"),
        )
        .arg(
            Arg::new("loop_interval")
                .long("loop-interval")
                .help(tr("cli.loop_interval"))
                .default_value("1"),
        )
        .arg(
            Arg::new("retry_interval")
                .long("retry-interval")
                .help(tr("cli.retry_interval"))
                .default_value("5"),
        )
        // Attachment options
        .arg(
            Arg::new("attachment")
                .long("attachment")
                .help(tr("cli.attachment")),
        )
        .arg(
            Arg::new("attachment_dir")
                .long("attachment-dir")
                .help(tr("cli.attachment_dir")),
        )
        // Template options
        .arg(
            Arg::new("subject_template")
                .long("subject-template")
                .help(tr("cli.subject_template")),
        )
        .arg(
            Arg::new("text_template")
                .long("text-template")
                .help(tr("cli.text_template")),
        )
        .arg(
            Arg::new("html_template")
                .long("html-template")
                .help(tr("cli.html_template")),
        )
        .arg(
            Arg::new("email_send_interval_ms")
                .long("email-send-interval-ms")
                .help(tr("cli.email_send_interval_ms"))
                .default_value("0"),
        )
        // Authentication options
        .arg(
            Arg::new("auth_mode")
                .long("auth-mode")
                .help(tr("cli.auth_mode"))
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("username")
                .long("username")
                .help(tr("cli.username")),
        )
        .arg(
            Arg::new("password")
                .long("password")
                .help(tr("cli.password")),
        )
        // TLS options
        .arg(
            Arg::new("use_tls")
                .long("use-tls")
                .help(tr("cli.use_tls"))
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("accept_invalid_certs")
                .long("accept-invalid-certs")
                .help(tr("cli.accept_invalid_certs"))
                .action(ArgAction::SetTrue),
        )
        // Logging options
        .arg(
            Arg::new("failed_emails_dir")
                .long("failed-emails-dir")
                .help(tr("cli.failed_emails_dir")),
        )
        .arg(
            Arg::new("log_file")
                .long("log-file")
                .help(tr("cli.log_file")),
        )
        // Language option (parsed early, before other args)
        .arg(
            Arg::new("lang")
                .long("lang")
                .help(tr("cli.lang"))
                .env("RSENDMAIL_LANG"),
        )
}

/// Detect language from command line args or environment
/// This is called before full CLI parsing to set the language
pub fn detect_language() -> Language {
    // First check environment variable
    if let Ok(lang_str) = std::env::var("RSENDMAIL_LANG") {
        if let Some(lang) = Language::from_str(&lang_str) {
            return lang;
        }
    }

    // Then check command line args for --lang
    let args: Vec<String> = std::env::args().collect();
    for i in 0..args.len() {
        if args[i] == "--lang" && i + 1 < args.len() {
            if let Some(lang) = Language::from_str(&args[i + 1]) {
                return lang;
            }
        }
        if args[i].starts_with("--lang=") {
            let lang_str = args[i].strip_prefix("--lang=").unwrap();
            if let Some(lang) = Language::from_str(lang_str) {
                return lang;
            }
        }
    }

    // Fall back to system language detection
    Language::from_system()
}

/// Parse CLI arguments and return Config
pub fn parse_args() -> Config {
    let matches = build_cli().get_matches();
    matches_to_config(&matches)
}

/// Convert ArgMatches to Config
fn matches_to_config(matches: &ArgMatches) -> Config {
    Config {
        smtp_server: matches.get_one::<String>("smtp_server").unwrap().clone(),
        port: matches
            .get_one::<String>("port")
            .unwrap()
            .parse()
            .unwrap_or(25),
        from: matches.get_one::<String>("from").unwrap().clone(),
        to: matches.get_one::<String>("to").unwrap().clone(),
        dir: matches.get_one::<String>("dir").cloned(),
        extension: matches.get_one::<String>("extension").unwrap().clone(),
        processes: matches.get_one::<String>("processes").unwrap().clone(),
        batch_size: matches
            .get_one::<String>("batch_size")
            .unwrap()
            .parse()
            .unwrap_or(1),
        smtp_timeout: matches
            .get_one::<String>("smtp_timeout")
            .unwrap()
            .parse()
            .unwrap_or(30),
        log_level: matches.get_one::<String>("log_level").unwrap().clone(),
        keep_headers: matches.get_flag("keep_headers"),
        anonymize_emails: matches.get_flag("anonymize_emails"),
        anonymize_domain: matches
            .get_one::<String>("anonymize_domain")
            .unwrap()
            .clone(),
        modify_headers: matches.get_flag("modify_headers"),
        r#loop: matches.get_flag("loop"),
        repeat: matches
            .get_one::<String>("repeat")
            .unwrap()
            .parse()
            .unwrap_or(1),
        loop_interval: matches
            .get_one::<String>("loop_interval")
            .unwrap()
            .parse()
            .unwrap_or(1),
        retry_interval: matches
            .get_one::<String>("retry_interval")
            .unwrap()
            .parse()
            .unwrap_or(5),
        attachment: matches.get_one::<String>("attachment").cloned(),
        attachment_dir: matches.get_one::<String>("attachment_dir").cloned(),
        subject_template: matches.get_one::<String>("subject_template").cloned(),
        text_template: matches.get_one::<String>("text_template").cloned(),
        html_template: matches.get_one::<String>("html_template").cloned(),
        email_send_interval_ms: matches
            .get_one::<String>("email_send_interval_ms")
            .unwrap()
            .parse()
            .unwrap_or(0),
        auth_mode: matches.get_flag("auth_mode"),
        username: matches.get_one::<String>("username").cloned(),
        password: matches.get_one::<String>("password").cloned(),
        use_tls: matches.get_flag("use_tls"),
        accept_invalid_certs: matches.get_flag("accept_invalid_certs"),
        failed_emails_dir: matches.get_one::<String>("failed_emails_dir").cloned(),
        log_file: matches.get_one::<String>("log_file").cloned(),
    }
}
