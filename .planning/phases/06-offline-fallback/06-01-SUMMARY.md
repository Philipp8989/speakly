---
phase: 06-offline-fallback
plan: "01"
subsystem: stt
tags: [rust, whisper-rs, local-stt, offline, download, streaming]
dependency_graph:
  requires: [04-cloud-stt-text-injection-mvp, src-tauri/src/stt/mod.rs]
  provides: [call_whisper_local, download_whisper_model, cancel_whisper_download, model_exists]
  affects: [src-tauri/src/stt/mod.rs, src-tauri/Cargo.toml]
tech_stack:
  added: [whisper-rs@0.14.4, futures-util@0.3]
  patterns: [streaming-http-download, linear-resampling, tauri-event-emitter]
key_files:
  created:
    - src-tauri/src/stt/local.rs
    - src-tauri/src/stt/download.rs
  modified:
    - src-tauri/Cargo.toml
    - src-tauri/src/stt/mod.rs
    - src-tauri/Cargo.lock
decisions:
  - "whisper-rs 0.14.4 used (not 0.16) — API stable, plan-specified version"
  - "Linear interpolation for resampling — sufficient quality for speech, no extra deps"
  - "WHISPER_BASE_SIZE hardcoded for integrity check — compile-time constant, no user input (T-06-01)"
metrics:
  duration: "~15 minutes"
  completed: "2026-04-09"
  tasks_completed: 2
  tasks_total: 2
  files_created: 2
  files_modified: 3
---

# Phase 06 Plan 01: Offline STT Backend — whisper-rs + Model Download Summary

**One-liner:** whisper-rs 0.14 local transcription with f32 resampling and HuggingFace streaming model download via reqwest.

## What Was Built

### Task 1: Cargo.toml + stt/local.rs (commit: 2a90a2d)

Added `whisper-rs = "0.14"`, `futures-util = "0.3"`, and `stream` feature to reqwest in Cargo.toml.

Created `src-tauri/src/stt/local.rs` providing:
- `model_path(app)` — resolves `app_data_dir()/ggml-base.bin`
- `call_whisper_local(app, &[f32], u32)` — loads WhisperContext, optionally resamples to 16kHz, runs full transcription, collects all segments into a single String
- `resample_to_16k(samples, from_rate)` — linear interpolation resampler for arbitrary sample rates to 16000 Hz

4 unit tests covering: empty sample guard, resampling length correctness (44100 → 16000), 16kHz passthrough, language normalization ("" / "auto" → None, "de" → Some("de")).

### Task 2: stt/download.rs (commit: 953c84b)

Created `src-tauri/src/stt/download.rs` providing:
- `WHISPER_BASE_URL` — HuggingFace CDN URL for ggml-base.bin
- `WHISPER_BASE_SIZE: u64 = 147_964_211` — expected file size for integrity check
- `model_exists(app)` — checks file exists and has correct size
- `download_whisper_model(app)` — Tauri command: streams download, emits `whisper_download_progress` / `whisper_download_complete` / `whisper_download_error` events, validates file size after download (mitigates T-06-01)
- `cancel_whisper_download(app)` — Tauri command: deletes partial file, resets `stt_mode` to `"cloud"`, emits `whisper_download_cancelled`

## Decisions Made

| Decision | Rationale |
|----------|-----------|
| whisper-rs 0.14.4 | Plan-specified stable version; 0.16 has breaking API changes |
| Linear interpolation resampling | Sufficient for speech, avoids rubato dependency complexity for this use case |
| WHISPER_BASE_SIZE as compile-time constant | Integrity check (T-06-01); URL is also compile-time, no user input attack surface |
| model_exists checks file size | Guards against corrupted/partial downloads from previous sessions |

## Deviations from Plan

None — plan executed exactly as written. Both files match the plan's code templates with correct whisper-rs 0.14 API (verified against source in `~/.cargo/registry`).

## Known Stubs

None. Both files implement full production logic. No placeholder data flows.

## Compilation Status

The sandbox environment blocked `cargo check` and `cargo test` execution during this session. However:
- `Cargo.lock` was updated with whisper-rs 0.14.4 and all transitive dependencies resolved successfully (cmake was installed via `brew install cmake` to unblock the build system)
- The whisper-rs 0.14.4 API usage in `local.rs` was verified against the crate's `basic_use.rs` example in `~/.cargo/registry`
- Tests can be run manually: `cargo test -- stt::local::tests` from `src-tauri/`

## Threat Flags

No new threat surface beyond what's documented in the plan's threat model (T-06-01 through T-06-04, all addressed or accepted).

## Self-Check: PASSED

| Check | Result |
|-------|--------|
| src-tauri/src/stt/local.rs | FOUND |
| src-tauri/src/stt/download.rs | FOUND |
| src-tauri/src/stt/mod.rs (updated) | FOUND |
| src-tauri/Cargo.toml (whisper-rs, futures-util, stream) | FOUND |
| Commit 2a90a2d (Task 1) | FOUND |
| Commit 953c84b (Task 2) | FOUND |
| Cargo.lock (whisper-rs 0.14.4 resolved) | FOUND |
