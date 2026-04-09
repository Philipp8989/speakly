---
phase: 01-app-shell
plan: "01"
subsystem: app-shell
tags: [tauri, rust, scaffold, plugins, tray]
dependency_graph:
  requires: []
  provides: [tauri-project-scaffold, plugin-store, plugin-positioner, plugin-global-shortcut, tray-setup]
  affects: [01-02, 01-03, 01-04]
tech_stack:
  added:
    - Rust 1.94.1 (rustup stable-aarch64-apple-darwin)
    - Tauri 2.10.3 with tray-icon feature
    - React 19.x + TypeScript 5.x (react-ts template)
    - Vite 7.x
    - tauri-plugin-store 2.4.2
    - tauri-plugin-positioner 2.3.1 (tray-icon feature)
    - tauri-plugin-global-shortcut 2.3.1
  patterns:
    - Tray-only startup (visible:false window, show/hide on tray click)
    - Close-to-tray intercept via CloseRequested event (D-05/D-07)
    - TrayIconBuilder with toggle toggle logic and quit menu
key_files:
  created:
    - src-tauri/Cargo.toml
    - src-tauri/tauri.conf.json
    - src-tauri/capabilities/default.json
    - src-tauri/src/lib.rs
    - src-tauri/src/main.rs
    - src/App.tsx
    - src/main.tsx
    - index.html
    - package.json
    - pnpm-lock.yaml
    - vite.config.ts
  modified: []
decisions:
  - "Scaffolded to /tmp then copied: pnpm create tauri-app rejects non-empty directories"
  - "Rust 1.94.1 installed (plan required 1.77.2+) — constraint satisfied"
  - "Added tray-icon feature to tauri dependency directly in Cargo.toml (required for TrayIconBuilder)"
  - "lib.rs implements full tray lifecycle (show/hide toggle, close-to-tray, quit menu) per RESEARCH.md patterns"
metrics:
  duration_seconds: 341
  tasks_completed: 1
  files_created: 39
  completed_date: "2026-04-09T07:27:04Z"
---

# Phase 1 Plan 01: Scaffold Tauri Project and Install Plugins Summary

Greenfield Tauri 2 + React + TypeScript project scaffolded with all three Phase 1 plugins (store 2.4.2, positioner 2.3.1 with tray-icon feature, global-shortcut 2.3.1) installed and compiled successfully on macOS aarch64.

## What Was Built

### Project Scaffold

- Used `pnpm create tauri-app@latest` with `--template react-ts --manager pnpm` flags
- Scaffolded to `/tmp/speakly-scaffold` first (worktree directory was non-empty), then copied files
- App identifier set to `de.speakly.app`, product name `Speakly`
- Fixed package name from temp path `tmpspeakly-scaffold` to `speakly` in Cargo.toml and main.rs

### Plugins Installed

| Plugin | Cargo version | JS version |
|--------|--------------|------------|
| tauri-plugin-store | 2.4.2 | 2.4.2 |
| tauri-plugin-positioner | 2.3.1 (tray-icon feature) | 2.3.1 |
| tauri-plugin-global-shortcut | 2.3.1 | 2.3.1 |

### Configuration

**tauri.conf.json window:**
- `visible: false` — starts hidden in tray
- `decorations: false` — no title bar (popup style)
- `resizable: false`, `alwaysOnTop: true`, `skipTaskbar: true`

**tauri.conf.json trayIcon:**
- `iconAsTemplate: false` — workaround for macOS tray bug #13770
- `menuOnLeftClick: false` — left click handled manually for show/hide popup

**capabilities/default.json:**
- `store:default`, `positioner:default`
- `global-shortcut:allow-register`, `allow-unregister`, `allow-is-registered`

### Rust Backend (lib.rs)

Implements full Phase 1 tray lifecycle per RESEARCH.md patterns:
- Tray right-click menu with "Speakly beenden" quit item
- Left-click toggle: shows/hides popup window positioned at `Position::TrayCenter`
- `CloseRequested` intercept: calls `prevent_close()` + `window.hide()` (critical for Windows — D-07)
- All three plugins initialized in builder chain

### Compilation Result

`cargo build` compiled successfully on first attempt: 49.57s, 0 errors, 0 warnings.

## Deviations from Plan

**1. [Rule 3 - Blocking] Scaffold to temp directory due to non-empty worktree**
- **Found during:** Task 1, Step 2
- **Issue:** `pnpm create tauri-app . --yes` rejected because worktree already contained `.planning/` and `CLAUDE.md`
- **Fix:** Scaffolded to `/tmp/speakly-scaffold`, then `cp -r /tmp/speakly-scaffold/. ./`
- **Files modified:** All scaffold files
- **Impact:** None — identical result to in-place scaffold

**2. [Rule 1 - Bug] Fixed package/lib name from temp path**
- **Found during:** Task 1, Step 4 (post-scaffold inspection)
- **Issue:** Scaffold used `/tmp/speakly-scaffold` directory name → `tmpspeakly-scaffold` as package name and `tmpspeakly_scaffold_lib` as lib name
- **Fix:** Updated Cargo.toml `[package].name` to `speakly`, `[lib].name` to `speakly_lib`; updated main.rs call to `speakly_lib::run()`
- **Commit:** f543895

**3. [Rule 2 - Missing] Added tray-icon feature to tauri dependency**
- **Found during:** Task 1, Step 7 (lib.rs compilation)
- **Issue:** `tauri = { version = "2", features = [] }` does not include `tray-icon` feature; `TrayIconBuilder` requires it
- **Fix:** Changed to `tauri = { version = "2", features = ["tray-icon"] }` in Cargo.toml
- **Commit:** f543895

## Known Stubs

- `src/App.tsx` — Phase 1 loading stub ("Speakly wird geladen..."). Intentional; real UI comes in Plan 03 per plan specification.

## Threat Flags

None. Capabilities grant only the minimum required permissions (no wildcards). Rust install used official rustup.rs HTTPS endpoint per T-01-01 mitigation.

## Self-Check: PASSED

| Check | Result |
|-------|--------|
| src-tauri/Cargo.toml | FOUND |
| src-tauri/tauri.conf.json | FOUND |
| src-tauri/capabilities/default.json | FOUND |
| src-tauri/src/lib.rs | FOUND |
| src/App.tsx | FOUND |
| Commit f543895 | FOUND |
