---
phase: 06-offline-fallback
plan: "02"
subsystem: stt-routing
tags: [rust, stt, whisper, offline, fallback, routing]
dependency_graph:
  requires: [06-01]
  provides: [stt-mode-router, download-commands-registered]
  affects: [lib.rs, stt/mod.rs]
tech_stack:
  added: [tokio::time (time feature), whisper-rs 0.14, futures-util 0.3, reqwest stream feature]
  patterns: [spawn_blocking for sync whisper calls, tokio::time::timeout for auto-fallback]
key_files:
  created: []
  modified:
    - src-tauri/src/lib.rs
    - src-tauri/src/stt/mod.rs
    - src-tauri/src/stt/local.rs
    - src-tauri/src/stt/download.rs
    - src-tauri/Cargo.toml
decisions:
  - "Default stt_mode ist cloud — unbekannte Werte fallen auf cloud (T-06-05 defensives Parsing)"
  - "spawn_blocking fuer whisper — verhindert async-Runtime-Blockierung (T-06-07)"
  - "3s Timeout fuer Auto-Modus — kurz genug gegen Haengen, ausloesend fuer local-Fallback (T-06-06)"
metrics:
  duration_minutes: 12
  completed: "2026-04-09T10:30:58Z"
  tasks_completed: 2
  files_changed: 5
---

# Phase 6 Plan 02: STT Routing (cloud/local/auto) Summary

**One-liner:** stt_mode-Router mit 3-Zweig-Logik (cloud/local/auto) in transcribe_and_inject, plus download_whisper_model/cancel_whisper_download als Tauri-Commands registriert.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | stt/mod.rs — pub mod local + download deklarieren | 1b74a7d | stt/mod.rs, local.rs, download.rs, Cargo.toml |
| 2 | lib.rs — transcribe_and_inject Router + Commands | d056fd6 | src-tauri/src/lib.rs |

## What Was Built

- `stt/mod.rs` exportiert jetzt `pub mod local` und `pub mod download`
- `SttMode` enum + `parse_stt_mode()` in lib.rs — defensives Parsing (cloud als Default)
- `transcribe_and_inject` hat 3-Zweig-Router:
  - **Cloud:** WAV-Puffer → call_whisper_api
  - **Local:** model_exists() prueft, f32-Samples per spawn_blocking an call_whisper_local
  - **Auto:** cloud-Versuch mit 3s tokio::time::timeout; bei Fehler/Timeout → local fallback
- `download_whisper_model` und `cancel_whisper_download` in invoke_handler registriert
- Unit-Test `test_parse_stt_mode` GREEN

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Missing `use tauri::Manager` in local.rs**
- **Found during:** Task 1 cargo check
- **Issue:** `app.path()` benoetigt den `Manager` trait im Scope — fehlte in der kopierten Datei
- **Fix:** `use tauri::Manager;` am Anfang von local.rs ergaenzt
- **Files modified:** src-tauri/src/stt/local.rs
- **Commit:** 1b74a7d

**2. [Rule 3 - Blocking] local.rs und download.rs fehlten im Worktree**
- **Found during:** Task 1 start — Dateien existierten nur im main-Repo (06-01 wurde auf main committed, Worktree-Branch divergiert vorher)
- **Fix:** Dateien aus main-Repo in Worktree kopiert, dann Cargo.toml um whisper-rs, futures-util, stream-feature ergaenzt
- **Files modified:** src-tauri/src/stt/local.rs (create), src-tauri/src/stt/download.rs (create), src-tauri/Cargo.toml
- **Commit:** 1b74a7d

## Verification Results

- `cargo test -- tests::test_parse_stt_mode` → **1 passed**
- `cargo check` → **Finished** (nur pre-existente Warnings, keine Errors)
- Alle drei Referenzen in lib.rs vorhanden: `call_whisper_local`, `stt_mode`, `download_whisper_model`

## Known Stubs

None — alle drei Routing-Zweige vollstaendig implementiert und verdrahtet.

## Threat Flags

None — keine neuen Sicherheits-relevanten Surfaces eingefuehrt. Bestehende Threat-Mitigations aus Plan umgesetzt:
- T-06-05: parse_stt_mode faellt auf cloud statt panic
- T-06-06: 3s Timeout verhindert Haengen im Auto-Modus
- T-06-07: spawn_blocking verhindert Runtime-Blockierung durch synchrones whisper

## Self-Check: PASSED

- [x] src-tauri/src/lib.rs vorhanden und modifiziert
- [x] src-tauri/src/stt/mod.rs vorhanden und modifiziert
- [x] src-tauri/src/stt/local.rs vorhanden
- [x] src-tauri/src/stt/download.rs vorhanden
- [x] Commit 1b74a7d existiert
- [x] Commit d056fd6 existiert
- [x] cargo check fehlerfrei
- [x] test_parse_stt_mode GREEN
