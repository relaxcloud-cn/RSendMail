# RSendMail Architecture

English | [简体中文](ARCHITECTURE_zh.md) | [繁體中文](ARCHITECTURE_zh-TW.md) | [日本語](ARCHITECTURE_ja.md)

This document describes the architecture and design of RSendMail, a high-performance bulk email sending tool.

## Overview

RSendMail is a Rust-based application for testing and sending bulk emails via SMTP. It provides both CLI and GUI interfaces, sharing a common core library.

```
┌─────────────────────────────────────────────────────────┐
│                    Applications                          │
├──────────────────────┬──────────────────────────────────┤
│    rsendmail-cli     │         rsendmail-gui            │
│   (Command Line)     │      (Slint GUI)                 │
├──────────────────────┴──────────────────────────────────┤
│                   rsendmail-core                         │
│            (Email Sending Engine)                        │
├─────────────────────────────────────────────────────────┤
│                   rsendmail-i18n                         │
│            (Internationalization)                        │
└─────────────────────────────────────────────────────────┘
```

## Project Structure

```
RSendMail/
├── Cargo.toml                      # Workspace configuration
├── crates/
│   ├── rsendmail-i18n/             # Internationalization support
│   │   ├── src/lib.rs              # Language enum, tr() functions
│   │   └── locales/                # YAML translation files
│   │       ├── en-US.yml           # English (fallback)
│   │       ├── zh-CN.yml           # Simplified Chinese
│   │       ├── zh-TW.yml           # Traditional Chinese
│   │       └── ja-JP.yml           # Japanese
│   │
│   ├── rsendmail-core/             # Core library
│   │   └── src/
│   │       ├── lib.rs              # Library exports
│   │       ├── config.rs           # Configuration structure
│   │       ├── mailer.rs           # Email sending engine (~1800 lines)
│   │       ├── stats.rs            # Statistics collection
│   │       └── anonymizer.rs       # Email anonymization
│   │
│   ├── rsendmail-cli/              # CLI application
│   │   └── src/
│   │       ├── main.rs             # Entry point, loop control
│   │       ├── args.rs             # CLI argument parsing (clap builder)
│   │       └── logging.rs          # Log initialization
│   │
│   └── rsendmail-gui/              # GUI application
│       ├── src/
│       │   ├── main.rs             # GUI entry point
│       │   └── i18n.rs             # GUI-specific i18n
│       ├── ui/
│       │   └── app.slint           # UI definition
│       └── fonts/                  # Custom fonts
│
├── assets/
│   └── screenshots/                # GUI screenshots
│
└── docs/
    └── ARCHITECTURE.md             # This file
```

## Crate Dependencies

```
rsendmail-cli ──┬──► rsendmail-core ──► rsendmail-i18n
                │
                └──► rsendmail-i18n

rsendmail-gui ──┬──► rsendmail-core ──► rsendmail-i18n
                │
                └──► (GUI has its own i18n via HashMap)
```

## Core Components

### 1. rsendmail-i18n

Shared internationalization module using `rust-i18n` library.

**Features:**
- 4 supported languages: English, Simplified Chinese, Traditional Chinese, Japanese
- Language detection from environment variables and system locale
- Translation functions: `tr(key)` and `tr_with_args(key, args)`
- YAML-based translation files (~250 keys per language)

**Language Detection Priority:**
1. `--lang` CLI argument
2. `RSENDMAIL_LANG` environment variable
3. `LANG` / `LC_ALL` environment variables
4. macOS `AppleLocale` (on macOS)
5. Default to English

### 2. rsendmail-core

The core email sending engine, shared by CLI and GUI.

**Modules:**

#### config.rs
- `Config` struct with 30+ configuration options
- Serde serialization support for save/load
- Default values for all optional fields
- `ProcessMode` enum (Auto / Fixed)

#### mailer.rs (~1800 lines)
The main email sending logic with three operating modes:

1. **EML Mode** (`--dir`)
   - Reads EML files from a directory
   - Supports batch sending in single SMTP session
   - Multi-process parallel sending

2. **Attachment Mode** (`--attachment`)
   - Sends a single file as email attachment
   - Auto-detects MIME type
   - Template support for subject/body

3. **Attachment Directory Mode** (`--attachment-dir`)
   - Sends each file in directory as separate email
   - Same template support as single attachment

**Connection Handling:**
- Plain text connection (port 25)
- STARTTLS (port 587)
- Implicit TLS (port 465)
- SMTP authentication (username/password)
- Connection timeout and retry logic
- Connection problem detection (421 errors, broken pipe)

#### stats.rs
- `Stats` struct for tracking:
  - Email count (total, success, failed)
  - Parse/send durations
  - Error classification with file lists
  - QPS (queries per second) calculation
- Implements `Display` trait for formatted output

#### anonymizer.rs
- Replaces email addresses with random strings
- Maintains consistency (same email → same replacement)
- Uses HashMap for caching

### 3. rsendmail-cli

Command-line interface application.

**Features:**
- 30+ command-line options
- Localized `--help` output
- Graceful shutdown (Ctrl+C handling)
- Loop and repeat modes
- Optional log file output
- Failed email saving

**Architecture:**
- Uses clap builder pattern (not derive) for runtime i18n
- Early language detection before CLI parsing
- Tokio async runtime

### 4. rsendmail-gui

Graphical user interface using Slint framework.

**Features:**
- Visual SMTP configuration
- Three sending modes with mode-specific UI
- Real-time progress and statistics
- Log viewing with export
- Configuration save/load (JSON)
- Language switcher
- Custom logger for dual output (terminal + GUI)

**UI Components:**
- Main window with tabbed sections
- SMTP server settings panel
- Send mode selector
- Advanced options panel
- Statistics display
- Log viewer

## Data Flow

### CLI Flow
```
main.rs
  │
  ├─► detect_language() ──► set_language()
  │
  ├─► parse_args() ──► Config
  │
  ├─► init_logging()
  │
  ├─► Mailer::new(config)
  │
  └─► Loop:
        │
        ├─► mailer.send_all_with_cancel(running)
        │     │
        │     ├─► EML mode: collect_email_files() → send_fixed_mode()
        │     ├─► Attachment mode: send_attachment_with_cancel()
        │     └─► Attachment-dir mode: send_attachment_dir_with_cancel()
        │
        ├─► Accumulate Stats
        │
        └─► Wait for next iteration (if loop/repeat)
```

### GUI Flow
```
main.rs
  │
  ├─► init_logger() (GuiLogger)
  │
  ├─► AppWindow::new()
  │
  ├─► setup_i18n()
  │
  ├─► setup_callbacks()
  │     │
  │     ├─► on_start_send() ──► spawn async task
  │     │     │
  │     │     └─► Mailer::send_all_with_cancel()
  │     │           │
  │     │           └─► Send events via mpsc channel
  │     │
  │     ├─► on_stop_send() ──► set running = false
  │     │
  │     ├─► on_browse_*() ──► file dialogs
  │     │
  │     └─► on_save/load_config() ──► JSON serialize/deserialize
  │
  └─► app.run()
```

## Key Dependencies

| Dependency | Purpose |
|------------|---------|
| tokio | Async runtime |
| mail-send | SMTP client |
| mail-parser | EML file parsing |
| mail-builder | Email construction |
| clap | CLI argument parsing |
| slint | GUI framework |
| rust-i18n | Internationalization |
| serde | Configuration serialization |
| walkdir | Directory traversal |
| infer | MIME type detection |

## Error Handling

- `anyhow::Result` for application-level errors
- `Stats.increment_error()` for per-email error tracking
- Error classification by type (connection, auth, send, parse)
- Failed email file saving for later analysis

## Thread Safety

- `Arc<AtomicBool>` for graceful shutdown signaling
- `Arc<Mutex<...>>` for shared state in GUI logger
- Tokio channels for GUI event communication
- Per-process stats in multi-process mode

## Configuration

The `Config` struct supports:
- Direct field access in code
- JSON serialization for GUI save/load
- CLI argument parsing
- Default values for all optional fields

## Future Considerations

- Additional email providers beyond SMTP
- Email template library
- Scheduling and queue management
- Web-based dashboard
- Plugin system for custom processors
