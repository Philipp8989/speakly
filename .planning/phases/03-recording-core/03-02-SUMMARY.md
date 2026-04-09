---
phase: 03-recording-core
plan: "02"
subsystem: recording-integration
tags: [rust, tauri, hotkey, recording, tray]
dependency_graph:
  requires: ["03-01"]
  provides: ["03-03", "03-04"]
  affects: ["src-tauri/src/lib.rs", "src-tauri/src/tray.rs", "src/App.tsx"]
tech_stack:
  added: []
  patterns:
    - "Hold-vs-toggle detection via Arc<Mutex<Option<Instant>>> shared between handler closures"
    - "start_capture / stop_capture as thin public wrappers over capture thread + accumulator thread"
    - "RecordingState as Tauri managed state, accessed via app.state::<RecordingState>()"
key_files:
  created: []
  modified:
    - src-tauri/src/tray.rs
    - src-tauri/src/recording/mod.rs
    - src-tauri/src/lib.rs
    - src/App.tsx
decisions:
  - "Added start_capture/stop_capture to recording/mod.rs as public wrappers (plan interfaces referenced them but 03-01 only built start_capture_thread in capture.rs)"
  - "VAD threshold in stop_capture hardcoded to 0.01 — ConfigStore value used at runtime in Plan 03-04 when STT dispatch reads it"
  - "Pre-existing TS error in useSettings.ts (missing defaults: {}) is out of scope — deferred"
metrics:
  duration: "~25 minutes"
  completed: "2026-04-09"
  tasks_completed: 2
  files_changed: 4
---

# Phase 03 Plan 02: Hotkey Wiring and Recording Integration Summary

## One-liner

Wired RecordingState into Tauri managed state, replaced JS hotkey stub with Rust with_handler implementing 300ms hold-vs-toggle detection, and added start_capture/stop_capture public API to the recording module.

## What Was Built

### Task 1: Rust backend wiring (c541d9b)

**tray.rs:** Changed `TrayIconBuilder::new()` to `TrayIconBuilder::with_id("main")`. The icon is now retrievable via `app.tray_by_id("main")` for runtime state swap (Plan 03-03).

**recording/mod.rs:** Added two public functions missing from Plan 03-01:
- `start_capture(state, device_name, app_handle)` — resets buffer/flags, spawns capture thread + accumulator thread, emits `recording_state_changed { recording: true }`
- `stop_capture(state, app_handle) -> Option<Vec<u8>>` — sets stop flags, waits 100ms for final frames, runs VAD, encodes WAV, emits `recording_stopped` or `recording_discarded`

**lib.rs:**
- `.manage(RecordingState::default())` registered before setup
- Global shortcut plugin replaced with `with_handler` closure implementing:
  - Pressed: record `Instant::now()` in `Arc<Mutex<Option<Instant>>>`, start capture if not recording; macOS key-repeat guard (`already_pressed` check)
  - Released: if elapsed > 300ms → hold mode, stop capture; else toggle mode, recording continues
- `toggle_recording` and `stop_recording_hold` Tauri commands registered in invoke_handler
- ConfigStore Phase 3 defaults: `silence_timeout_ms = 3000`, `vad_rms_threshold = 0.01`

### Task 2: Frontend cleanup (474a37b)

**App.tsx:**
- Removed `import { register } from '@tauri-apps/plugin-global-shortcut'`
- Removed `registerHotkey()` function and its call in useEffect
- Added `import { listen } from '@tauri-apps/api/event'`
- Added `recording_state_changed` and `recording_discarded` event listeners in main-window useEffect with proper cleanup (`unlisteners.forEach(ul => ul())`)

## Verification Results

| Check | Result |
|-------|--------|
| `cargo check` errors | 0 |
| `tray.rs` contains `with_id("main")` | PASS |
| `lib.rs` has `.manage(RecordingState::default())` | PASS |
| `lib.rs` has `with_handler` hotkey plugin | PASS |
| `lib.rs` has `toggle_recording` + `stop_recording_hold` in invoke_handler | PASS |
| `lib.rs` has `silence_timeout_ms` + `vad_rms_threshold` defaults | PASS |
| `App.tsx` grep count for `registerHotkey\|plugin-global-shortcut` | 0 |
| `App.tsx` has `listen` import | PASS |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing functionality] Added start_capture/stop_capture to recording/mod.rs**
- **Found during:** Task 1, before editing lib.rs
- **Issue:** Plan interfaces specified `pub fn start_capture(...)` and `pub fn stop_capture(...)` in `recording/mod.rs`, but Plan 03-01 only built `start_capture_thread` in `capture.rs`. lib.rs commands couldn't compile without these wrappers.
- **Fix:** Added both functions to mod.rs. `start_capture` spawns capture thread + accumulator thread; `stop_capture` stops both, runs VAD, encodes WAV, emits events.
- **Files modified:** `src-tauri/src/recording/mod.rs`
- **Commit:** c541d9b

**2. [Rule 1 - Bug] Rust cast precedence error in mod.rs**
- **Found during:** Task 1, first `cargo check`
- **Issue:** `buffer.len() as f32 / sample_rate as f32 < 0.5` — Rust parsed `as f32 <` as generic argument syntax
- **Fix:** Changed to `/ (sample_rate as f32) < 0.5`
- **Files modified:** `src-tauri/src/recording/mod.rs`
- **Commit:** c541d9b

## Known Stubs

None. Recording event listeners in App.tsx log to console — Plan 03-03 will wire them to UI state. This is intentional scaffolding, not a stub blocking plan goals.

## Deferred Items

- Pre-existing TypeScript error in `src/hooks/useSettings.ts` line 32: `defaults` property missing in `StoreOptions`. Out of scope — not introduced by this plan.

## Threat Flags

None. No new network endpoints, auth paths, or trust boundary changes introduced.

## Self-Check: PASSED

- `c541d9b` exists: confirmed via `git log`
- `474a37b` exists: confirmed via `git log`
- `src-tauri/src/tray.rs` modified: confirmed
- `src-tauri/src/recording/mod.rs` modified: confirmed
- `src-tauri/src/lib.rs` modified: confirmed
- `src/App.tsx` modified: confirmed
