# RSendMail Architecture

English | [简体中文](ARCHITECTURE_zh.md) | [繁體中文](ARCHITECTURE_zh-TW.md) | [日本語](ARCHITECTURE_ja.md)

This document describes the current RSendMail architecture after consolidating the GUI onto Tauri.

## Overview

RSendMail has two user-facing applications:

- `rsendmail-cli` for command-line workflows
- `rsendmail-tauri` for the desktop GUI

Both entrypoints share the same `rsendmail-core` business logic so mail parsing, SMTP sending, retries, attachment handling, and statistics stay consistent across CLI and GUI.

```text
┌────────────────────────────────────────────────────────────┐
│                       Applications                         │
├───────────────────────────┬────────────────────────────────┤
│      rsendmail-cli        │        rsendmail-tauri        │
│      (Rust CLI)           │   (Tauri + Vue desktop GUI)   │
├───────────────────────────┴────────────────────────────────┤
│                      rsendmail-core                        │
│          Shared mailer, config, stats, anonymizer         │
├────────────────────────────────────────────────────────────┤
│                      rsendmail-i18n                        │
│           Shared translation source for Rust code         │
└────────────────────────────────────────────────────────────┘
```

## Project Structure

```text
RSendMail/
├── Cargo.toml
├── crates/
│   ├── rsendmail-i18n/
│   │   ├── locales/                # YAML translations
│   │   └── src/lib.rs
│   ├── rsendmail-core/
│   │   └── src/
│   │       ├── anonymizer.rs
│   │       ├── config.rs
│   │       ├── lib.rs
│   │       ├── mailer.rs
│   │       └── stats.rs
│   ├── rsendmail-cli/
│   │   └── src/
│   │       ├── args.rs
│   │       ├── logging.rs
│   │       └── main.rs
│   └── rsendmail-tauri/
│       ├── src/                    # Vue 3 + TypeScript frontend
│       ├── src-tauri/              # Rust Tauri shell and commands
│       ├── package.json
│       └── vite.config.ts
├── assets/
│   └── screenshots/
└── docs/
    └── ARCHITECTURE.md
```

## Dependency Relationships

```text
rsendmail-cli ─────► rsendmail-core ─────► rsendmail-i18n

rsendmail-tauri
  ├── frontend (Vue, vue-i18n, @tauri-apps/api)
  └── src-tauri (Rust shell) ─────► rsendmail-core
```

## Core Components

### 1. `rsendmail-i18n`

Shared Rust translation layer powered by `rust-i18n`.

- Stores YAML translation files under `crates/rsendmail-i18n/locales/`
- Handles language detection for Rust-side code
- Exposes `tr()` and `tr_with_args()` helpers

### 2. `rsendmail-core`

Shared mail engine used by both CLI and GUI.

- `config.rs`: serializable runtime configuration
- `mailer.rs`: EML sending, attachment sending, retries, SMTP session handling
- `stats.rs`: counters, durations, throughput, and reporting
- `anonymizer.rs`: deterministic email address anonymization

### 3. `rsendmail-cli`

Rust command-line application for automation and scripting.

- Parses localized CLI arguments with `clap`
- Initializes logging and optional log-file output
- Runs repeat/loop workflows on top of `Mailer`
- Preserves backward-compatible CLI behavior

### 4. `rsendmail-tauri`

Desktop GUI split into two layers:

- `src/`: Vue 3 + TypeScript + Vite UI
- `src-tauri/`: Rust Tauri shell exposing commands to the frontend

Key frontend responsibilities:

- Collect SMTP and send-mode configuration visually
- Start and stop sending through Tauri commands
- Listen for log, progress, and stats events from Rust
- Render multilingual desktop UI with `vue-i18n`

Key Rust-shell responsibilities:

- Receive `Config` from the frontend
- Reuse `rsendmail-core::Mailer` directly
- Forward runtime logs and progress to the GUI through Tauri events
- Maintain app-level running state with `Arc<AtomicBool>`

## Data Flow

### CLI Flow

```text
CLI args
  -> rsendmail-cli
  -> Config
  -> rsendmail-core::Mailer
  -> SMTP / filesystem operations
  -> logs + stats output
```

### GUI Flow

```text
Vue UI
  -> invoke("start_sending", Config)
  -> Tauri command handler
  -> rsendmail-core::Mailer
  -> emit log/progress/stats events
  -> Vue listeners update the desktop UI
```

## Key Dependencies

| Dependency | Purpose |
|------------|---------|
| `tokio` | Async runtime |
| `mail-send` | SMTP client |
| `mail-parser` | EML parsing |
| `mail-builder` | Attachment/body construction |
| `clap` | CLI argument parsing |
| `tauri` | Desktop app shell |
| `vue` | GUI component runtime |
| `vite` | Frontend build tool |
| `vue-i18n` | GUI translations |
| `serde` / `serde_json` | Shared configuration serialization |

## Concurrency and State

- `Arc<AtomicBool>` controls start/stop behavior for active send loops
- `Mutex<Option<AppHandle>>` lets the Rust logger forward messages to the GUI
- Tokio tasks keep the desktop UI responsive while mail sending runs

## Configuration Model

The `Config` struct is the contract shared by CLI and GUI.

- CLI builds it from parsed arguments
- Tauri GUI sends it from the Vue frontend into Rust commands
- Serde keeps save/load and interop predictable

## Maintenance Notes

- CLI behavior should remain stable unless explicitly changed
- GUI-only changes should live under `crates/rsendmail-tauri/`
- Mail-sending behavior should usually be implemented in `rsendmail-core`
