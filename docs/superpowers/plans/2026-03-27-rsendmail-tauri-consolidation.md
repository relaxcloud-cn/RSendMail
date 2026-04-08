# RSendMail Tauri Consolidation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remove the legacy Slint GUI, keep CLI behavior unchanged, and make `rsendmail-tauri` the only supported GUI and release artifact name.

**Architecture:** Keep `rsendmail-core` as the shared business layer for both entrypoints. Preserve `rsendmail-cli` as-is, promote the Tauri app under `crates/rsendmail-tauri` to the only GUI implementation, and update release/docs/configuration so the repository no longer references `rsendmail-gui`.

**Tech Stack:** Rust workspace, Tauri 2, Vue 3, TypeScript, Vite, GitHub Actions

---

### Task 1: Workspace and GUI crate consolidation

**Files:**
- Modify: `Cargo.toml`
- Modify: `crates/rsendmail-tauri/src-tauri/Cargo.toml`
- Modify: `crates/rsendmail-tauri/src-tauri/src/main.rs`
- Modify: `crates/rsendmail-tauri/src-tauri/tauri.conf.json`
- Delete: `crates/rsendmail-gui/`

- [ ] Remove `crates/rsendmail-gui` from the workspace members list.
- [ ] Remove `slint` and `rfd` workspace dependencies that are only needed by the deleted GUI crate.
- [ ] Rename the Tauri Rust package/lib/bin-facing metadata from template placeholders to `rsendmail-tauri`.
- [ ] Update Tauri product/window naming so bundled apps and desktop title use `rsendmail-tauri`.
- [ ] Delete the legacy Slint GUI crate directory once no remaining workspace references exist.

### Task 2: Documentation and release pipeline cleanup

**Files:**
- Modify: `README.md`
- Modify: `README_zh.md`
- Modify: `README_zh-TW.md`
- Modify: `README_ja.md`
- Modify: `CLAUDE.md`
- Modify: `docs/ARCHITECTURE.md`
- Modify: `docs/ARCHITECTURE_zh.md`
- Modify: `docs/ARCHITECTURE_zh-TW.md`
- Modify: `docs/ARCHITECTURE_ja.md`
- Modify: `.github/workflows/release.yml`

- [ ] Replace all `rsendmail-gui` references with `rsendmail-tauri`.
- [ ] Update architecture/docs to describe the GUI as Tauri + Vue rather than Slint.
- [ ] Keep CLI positioning intact and document the shared `rsendmail-core` relationship.
- [ ] Change release artifact names and build commands from `rsendmail-gui` to `rsendmail-tauri`.
- [ ] Remove old GUI naming from usage examples and download tables without adding compatibility aliases.

### Task 3: Verification

**Files:**
- Verify only

- [ ] Run a targeted search to confirm no intentional repo references to `rsendmail-gui` or `slint` remain outside historical git data or vendored build output.
- [ ] Run `cargo check -p rsendmail-cli` to verify CLI still builds unchanged.
- [ ] Run `cargo check -p rsendmail-tauri` or the equivalent package path check for the Tauri Rust crate.
- [ ] Run the frontend production build for `crates/rsendmail-tauri` to verify the Vue/Vite app still compiles.
