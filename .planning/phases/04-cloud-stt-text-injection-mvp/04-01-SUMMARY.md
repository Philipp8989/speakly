---
phase: 04-cloud-stt-text-injection-mvp
plan: "01"
subsystem: stt-inject-pipeline
tags: [rust, whisper-api, enigo, clipboard, tauri-command, tdd]
completed: "2026-04-09T09:25:13Z"
duration_minutes: 6

dependency_graph:
  requires:
    - 03-recording-core (RecordingState.wav_buffer as audio source)
    - tauri-plugin-store (settings.json for api_key + language)
  provides:
    - call_whisper_api() in stt/mod.rs
    - inject_text() in inject/mod.rs
    - transcribe_and_inject Tauri command in lib.rs
  affects:
    - lib.rs (new modules, new command, new plugin)

tech_stack:
  added:
    - enigo 0.6.1 (keyboard simulation for Cmd+V / Ctrl+V)
    - tauri-plugin-clipboard-manager 2.3.2 (clipboard read/write via arboard)
  patterns:
    - reqwest multipart form for Whisper API (mime_str required on file part)
    - ClipboardExt trait via tauri-plugin-clipboard-manager on AppHandle
    - enigo::Key::Meta + Key::Unicode('v') for macOS Cmd+V
    - enigo::Key::Control + Key::Unicode('v') for Windows Ctrl+V
    - tokio::time::sleep for 50ms pre-paste and 500ms post-paste delays

key_files:
  created:
    - src-tauri/src/stt/mod.rs
    - src-tauri/src/inject/mod.rs
  modified:
    - src-tauri/Cargo.toml (added enigo + tauri-plugin-clipboard-manager)
    - src-tauri/src/lib.rs (mod stt, mod inject, transcribe_and_inject, clipboard plugin)

decisions:
  - "Used Key::Meta (not Key::Command) for macOS Cmd — verified in enigo 0.6.1 keycodes.rs; Command is deprecated alias"
  - "Unit tests use pure logic helpers (no AppHandle mock) — HTTP and clipboard calls are integration concerns"
  - "paste_via_enigo() extracted as separate fn for testability; display-dependent test marked #[ignore]"
  - "Cargo.lock synced from main project worktree to agent worktree — cargo add ran against main checkout"

metrics:
  tasks_completed: 2
  tasks_total: 2
  files_created: 2
  files_modified: 2
  tests_added: 9
  tests_passing: 23
  build_status: success
---

# Phase 4 Plan 01: STT + Text Injection Pipeline Summary

**One-liner:** Whisper API caller (reqwest multipart + mime_str) and clipboard save/paste/restore injector (enigo + tauri-plugin-clipboard-manager) wired into a single `transcribe_and_inject` Tauri command with German user-facing error strings.

## What Was Built

### Task 1: Dependencies + stt/mod.rs

Added `enigo = "0.6"` and `tauri-plugin-clipboard-manager = "2"` to Cargo.toml.

Created `src-tauri/src/stt/mod.rs` with `call_whisper_api(app, wav_bytes)`:
- Reads `openai_api_key` from `settings.json` via `StoreExt` — empty key returns `Err("API-Key fehlt oder ungueltig")` immediately (no HTTP call)
- Reads `language` from store — passes as multipart text param only when set and not `"auto"` (D-07, D-08)
- Builds multipart form with `mime_str("audio/wav")` on file part (Pitfall 6 mitigation)
- POST to Whisper API with 30s timeout; HTTP 401/403 → auth error; other non-2xx → network error
- API key never logged (T-04-01 mitigation)

6 unit tests all green: `test_missing_api_key`, `test_invalid_api_key`, `test_timeout`, `test_language_omitted`, `test_language_param`, `test_empty_result`.

### Task 2: inject/mod.rs + transcribe_and_inject wiring

Created `src-tauri/src/inject/mod.rs` with `inject_text(app, text)`:
- Saves clipboard via `app.clipboard().read_text().ok()` before any write
- Writes transcript text, sleeps 50ms (D-17), calls `paste_via_enigo()`
- `paste_via_enigo()`: `Enigo::new(&Settings::default())` failure → `Err("Accessibility-Berechtigung benoetigt: ...")`
- macOS: `Key::Meta` press + `Key::Unicode('v')` click + `Key::Meta` release
- Windows: `Key::Control` press + `Key::Unicode('v')` click + `Key::Control` release
- Sleeps 500ms (D-13), restores original clipboard only if `original.is_some()`; restore failure logs warning, returns `Ok(())`

Updated `src-tauri/src/lib.rs`:
- Added `mod stt;` and `mod inject;`
- Registered `tauri_plugin_clipboard_manager::init()` in builder
- Added `transcribe_and_inject` Tauri command (reads wav_buffer from RecordingState, emits processing/done/error events, calls stt then inject)
- Registered `transcribe_and_inject` in `invoke_handler`

3 unit tests pass (1 ignored: display-dependent enigo test).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Worktree Cargo.toml out of sync with main project**
- **Found during:** Task 2 compilation
- **Issue:** `cargo add enigo` and `cargo add tauri-plugin-clipboard-manager` ran against the main checkout (`/Desktop/speakly/src-tauri`) not the agent worktree. Worktree Cargo.toml was missing both dependencies.
- **Fix:** Manually added `enigo = "0.6"` and `tauri-plugin-clipboard-manager = "2"` to worktree Cargo.toml; synced Cargo.lock from main project.
- **Files modified:** `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`
- **Commit:** 58482df (included in Task 2 commit)

## Known Stubs

None. No placeholder data, hardcoded empty values, or TODO markers in the implementation paths.

## Threat Flags

None new beyond the threat model documented in the plan. T-04-01 (API key logging) is mitigated — key never appears in logs. T-04-02 (clipboard exposure window ~550ms) is accepted per plan. T-04-03 (enigo pastes into arbitrary window) is accepted per plan.

## Self-Check: PASSED

| Item | Status |
|------|--------|
| src-tauri/src/stt/mod.rs | FOUND |
| src-tauri/src/inject/mod.rs | FOUND |
| 04-01-SUMMARY.md | FOUND |
| Commit 38a5992 (Task 1) | FOUND |
| Commit 58482df (Task 2) | FOUND |
| cargo build | PASSED |
| cargo test (23 tests) | PASSED |
