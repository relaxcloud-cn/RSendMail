# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

RSendMail is a high-performance Rust CLI tool for bulk email sending and testing. It supports batch processing of EML files, sending files as attachments, and multi-threaded SMTP operations.

## Project Structure (Cargo Workspace)

```
RSendMail/
├── Cargo.toml                    # Workspace root configuration
├── crates/
│   ├── rsendmail-i18n/           # Shared internationalization (i18n) support
│   │   ├── src/lib.rs            # Language enum, tr() functions
│   │   └── locales/              # Translation files (YAML)
│   │       ├── en-US.yml         # English translations
│   │       ├── zh-CN.yml         # Simplified Chinese
│   │       ├── zh-TW.yml         # Traditional Chinese
│   │       └── ja-JP.yml         # Japanese
│   ├── rsendmail-core/           # Core library (shared by CLI and GUI)
│   │   └── src/
│   │       ├── lib.rs            # Library entry point
│   │       ├── config.rs         # Configuration structure (no CLI dependencies)
│   │       ├── mailer.rs         # Core email sending logic
│   │       ├── stats.rs          # Statistics collection
│   │       └── anonymizer.rs     # Email anonymization
│   ├── rsendmail-cli/            # CLI application
│   │   └── src/
│   │       ├── main.rs           # CLI entry point
│   │       ├── args.rs           # CLI argument parsing (clap builder pattern)
│   │       └── logging.rs        # Logging initialization
│   └── rsendmail-gui/            # GUI application (Slint-based)
│       └── src/
│           ├── main.rs           # GUI entry point
│           └── i18n.rs           # GUI-specific i18n helpers
├── rsendmail/                    # [DEPRECATED] Old single-crate structure
└── docs/
    ├── ARCHITECTURE.md           # Architecture design document
    └── GUI_DESIGN.md             # GUI functional design document
```

## Build Commands

```bash
# Build entire workspace
cargo build --workspace

# Build release version
cargo build --release --workspace

# Build only CLI
cargo build -p rsendmail-cli

# Build only core library
cargo build -p rsendmail-core

# Run tests
cargo test --workspace

# Run CLI with arguments
cargo run -p rsendmail-cli -- --help

# Docker build
docker build -t rsendmail .
```

## Architecture

### Core Library (`rsendmail-core`)

The core library contains all business logic, independent of CLI or GUI:

- **config.rs** - Configuration structure with serde support for serialization/deserialization
- **mailer.rs** - Core email sending logic with three operating modes
- **stats.rs** - Statistics collection for parse/send durations, error counts, QPS calculations
- **anonymizer.rs** - Email address anonymization with HashMap-based caching

### CLI Application (`rsendmail-cli`)

The CLI application provides command-line interface:

- **main.rs** - Entry point handling Tokio async runtime, loop/repeat iterations, graceful shutdown (Ctrl+C)
- **args.rs** - CLI argument parsing via clap with 30+ configuration options
- **logging.rs** - Logging initialization with optional file output

## Key Dependencies

- `mail-send` / `mail-parser` / `mail-builder` - SMTP client and email handling
- `tokio` - Async runtime (full features)
- `clap` - CLI argument parsing with builder pattern for runtime i18n (CLI only)
- `simplelog` - Logging with optional file output (CLI only)
- `serde` / `serde_json` - Configuration serialization (for future GUI)
- `rust-i18n` - Internationalization support with YAML translation files

## Three Operating Modes

1. **EML Mode**: `--dir ./emails` - Reads EML files and sends them in batches
2. **Attachment Mode**: `--attachment ./file.pdf` - Creates email with single attachment
3. **Attachment-Dir Mode**: `--attachment-dir ./docs` - Creates separate email for each file in directory

## Connection Handling

The mailer implements connection problem detection (421 errors, Broken pipe, timeouts) with automatic RSET commands and connection reset. Failed emails can be saved to `--failed-emails-dir` for later analysis.

## Internationalization (i18n)

The project supports 4 languages: English, Simplified Chinese, Traditional Chinese, and Japanese.

### i18n Architecture

- **rsendmail-i18n crate** - Shared i18n module using `rust-i18n` library
- Translation files in YAML format under `crates/rsendmail-i18n/locales/`
- Key functions: `tr(key)` for simple translations, `tr_with_args(key, args)` for parameterized translations

### Adding/Modifying Translations

1. Edit YAML files in `crates/rsendmail-i18n/locales/`:
   - `en-US.yml` - English (fallback)
   - `zh-CN.yml` - Simplified Chinese
   - `zh-TW.yml` - Traditional Chinese
   - `ja-JP.yml` - Japanese

2. Use placeholder format `%{name}` for variables in translations

3. Call translations using:
   ```rust
   use rsendmail_i18n::{tr, tr_with_args};

   // Simple translation
   let msg = tr("core.mailer.interrupted");

   // With parameters
   let msg = tr_with_args("core.mailer.found_files", &[("count", "10")]);
   ```

### Language Detection Priority

1. `--lang` CLI argument
2. `RSENDMAIL_LANG` environment variable
3. System locale (LANG, LC_ALL, or macOS AppleLocale)
4. Default to English

## Design Documents

- `docs/ARCHITECTURE.md` - Detailed architecture design for workspace refactoring
- `docs/GUI_DESIGN.md` - GUI functional design for future Slint-based GUI application
