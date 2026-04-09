---
phase: 01-app-shell
plan: "02"
subsystem: app-shell
tags: [tauri, rust, tray, positioner, close-to-tray, config-store]
dependency_graph:
  requires: [01-01]
  provides: [tray-module, close-to-tray-intercept, config-store-defaults, plugin-wiring]
  affects: [01-03, 01-04]
tech_stack:
  added: []
  patterns:
    - tray.rs module with TrayIconBuilder, positioned popup toggle, quit menu
    - mod tray delegation pattern in lib.rs
    - ConfigStore defaults written on first launch via StoreExt
    - CloseRequested intercept (prevent_close + hide) for macOS and Windows
key_files:
  created:
    - src-tauri/src/tray.rs
  modified:
    - src-tauri/src/lib.rs
decisions:
  - "tray.rs extracted as separate module (mod tray) — lib.rs stays thin, tray logic encapsulated"
  - "Manager import required in tray.rs for get_webview_window on AppHandle — RESEARCH.md pattern omitted this"
  - "ConfigStore defaults written Rust-side (StoreExt) not frontend-side — avoids race condition on first launch"
  - "prevent_close() applied universally (both macOS and Windows) per D-05 requirement"
metrics:
  duration_seconds: 102
  tasks_completed: 2
  files_created: 1
  files_modified: 1
  completed_date: "2026-04-09T07:31:00Z"
requirements:
  - APP-01
  - APP-06
---

# Phase 1 Plan 02: Tray Lifecycle and Close-to-Tray Backend Summary

Rust backend fully wired: `src-tauri/src/tray.rs` implements TrayIconBuilder with positioned popup toggle and quit menu; `src-tauri/src/lib.rs` delegates to `tray::setup_tray`, wires all three plugins, writes ConfigStore defaults, and intercepts CloseRequested to hide rather than quit on both macOS and Windows.

## What Was Built

### tray.rs

New module extracted from lib.rs scaffold with full tray logic:

- `TrayIconBuilder::new()` with `.icon()`, `.menu()`, `.show_menu_on_left_click(false)`
- Right-click menu: "Speakly beenden" (MenuItem id: "quit") triggering `app.exit(0)` (D-06)
- `on_tray_icon_event`: forwards all events to `tauri_plugin_positioner::on_tray_event` for correct positioning, then toggles popup on left-click-up via `window.move_window(Position::TrayCenter)` / `show()` / `hide()` (D-01)
- Public function signature: `pub fn setup_tray(app: &mut App) -> tauri::Result<()>`

### lib.rs

Replaced inline tray implementation with clean delegation pattern:

- `mod tray;` + `tray::setup_tray(app)?` in setup closure
- Plugin chain: `tauri_plugin_store`, `tauri_plugin_positioner::init()`, `tauri_plugin_global_shortcut`, `tauri_plugin_opener`
- ConfigStore defaults on first launch: `hotkey = "CommandOrControl+Shift+Space"`, `stt_mode = "cloud"`, `initialized = true` (D-08)
- `WindowEvent::CloseRequested` intercept: `api.prevent_close()` + `win_clone.hide()` — covers both platforms (D-05, D-07)

### Compilation

`cargo check` passed with zero errors on first attempt after the `Manager` import fix (see deviations).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Missing `Manager` import in tray.rs**
- **Found during:** Task 1 cargo check
- **Issue:** `app.get_webview_window("main")` called on `&AppHandle<_>` without `tauri::Manager` in scope; compiler error E0599
- **Fix:** Added `Manager` to the tauri import block in tray.rs: `use tauri::{..., App, Manager}`
- **Files modified:** `src-tauri/src/tray.rs`
- **Commit:** 6976164

## Output Notes (from plan spec)

- **cargo check first try:** Failed once due to missing `Manager` import; passed after one-line fix
- **API differences vs RESEARCH.md:** RESEARCH.md Pattern 2 imports were `use tauri::tray::{...}` at module level — correct, but omitted `Manager` which is required for `AppHandle::get_webview_window`. Added to tray.rs imports.
- **Close-to-tray macOS:** `prevent_close()` applied universally per D-05 (both platforms). No macOS-specific side-effects expected; Cmd+Q and tray quit menu remain the only exit paths.
- **settings.json location:** Written to Tauri app data directory — on macOS: `~/Library/Application Support/de.speakly.app/settings.json`

## Known Stubs

None — no UI rendering paths, no placeholder data. Tray module and lib.rs are purely behavioral.

## Threat Flags

None. No new network endpoints, auth paths, or file access patterns beyond what was already in the plan's threat model (T-02-01 through T-02-04).

## Self-Check: PASSED

| Check | Result |
|-------|--------|
| src-tauri/src/tray.rs | FOUND |
| src-tauri/src/lib.rs (mod tray present) | FOUND |
| Commit e3aa2f4 (tray.rs) | FOUND |
| Commit 6976164 (lib.rs + tray.rs fix) | FOUND |
| cargo check exit code 0 | PASSED |
