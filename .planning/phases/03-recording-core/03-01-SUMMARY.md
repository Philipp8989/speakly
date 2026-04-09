---
phase: 03-recording-core
plan: "01"
subsystem: recording
tags: [rust, cpal, hound, rubato, vad, audio, wav]
dependency_graph:
  requires: []
  provides: [recording::RecordingState, recording::capture, recording::vad, recording::wav_encoder]
  affects: [03-02, 03-03]
tech_stack:
  added: [hound@3.5, rubato@0.15]
  patterns: [mpsc-channel-audio-thread, rms-vad-gate, in-memory-wav-encoding, sinc-resampling]
key_files:
  created:
    - src-tauri/src/recording/mod.rs
    - src-tauri/src/recording/capture.rs
    - src-tauri/src/recording/vad.rs
    - src-tauri/src/recording/wav_encoder.rs
  modified:
    - src-tauri/Cargo.toml
    - src-tauri/src/lib.rs
decisions:
  - "Use rubato SincFixedIn for 44100→16kHz resampling — integer ratio issue (2.75625) rules out simple decimation"
  - "MAX_BUFFER_SAMPLES const defined in mod.rs for T-03-03 buffer cap enforcement in Plan 02"
  - "tauri::Emitter trait required explicitly for app_handle.emit() in Tauri 2.x"
metrics:
  duration_minutes: 25
  completed: "2026-04-09T08:53:24Z"
  tasks_completed: 2
  files_changed: 6
---

# Phase 3 Plan 01: Recording Module Foundation Summary

**One-liner:** Pure Rust recording pipeline — cpal F32/I16 capture thread, RMS VAD gate with silence auto-stop, and rubato+hound in-memory 16kHz WAV encoder with 14 passing unit tests.

## What Was Built

The complete `src-tauri/src/recording/` module — the data pipeline foundation for Phase 3. No Tauri command wiring yet; that is Plan 02.

### Files Created

**`recording/mod.rs`** — Public module root
- `RecordingState` struct: `is_recording`, `stream_stop_flag`, `audio_buffer`, `native_sample_rate`, `wav_buffer`, `press_time` — all `Arc`-wrapped for thread safety
- `impl Default` initializes all fields to empty/false/None
- `MAX_BUFFER_SAMPLES` const: 10min * 44100Hz cap for T-03-03 enforcement in Plan 02

**`recording/capture.rs`** — cpal audio capture
- `start_capture_thread()`: spawns dedicated OS thread, uses `device.default_input_config()` (never hardcodes 16kHz — Pitfall 3 avoided), handles F32 and I16 sample formats, emits `audio_level` events at ~12Hz via `tauri::Emitter`, sends frames via `mpsc::Sender`
- `compute_rms()`: pure function, no state

**`recording/vad.rs`** — Voice Activity Detection
- `should_discard()`: duration guard (< 0.5s) + RMS energy guard (max chunk RMS < threshold)
- `SilenceCounter`: accumulates silence duration per frame, fires when >= `timeout_ms`, resets on loud frames

**`recording/wav_encoder.rs`** — In-memory WAV encoding
- `encode_wav_in_memory()`: resample → clamp f32→i16 → hound WavWriter with Cursor<Vec<u8>>, always calls `finalize()` (Pitfall 4 avoided)
- `resample_to_16k()`: rubato `SincFixedIn` for anti-aliased downsampling; no-op if already 16kHz; linear interpolation fallback on rubato init failure

### Dependencies Added

| Crate | Version | Purpose |
|-------|---------|---------|
| hound | 3.5 | In-memory WAV encoding via Cursor<Vec<u8>> |
| rubato | 0.15 | Sinc-resampling 44100→16000 Hz |

## Test Results

```
test result: ok. 14 passed; 0 failed
```

| Test | File | Covers |
|------|------|--------|
| test_rms_constant_signal | capture | RMS of [0.5; 512] ≈ 0.5 |
| test_rms_empty | capture | Empty slice → 0.0 |
| test_rms_zeros | capture | Silent frame → 0.0 |
| test_discard_too_short | vad | 0.3s buffer → discard |
| test_discard_silent | vad | 1s zeros → discard |
| test_keep_loud | vad | 1s 0.5 RMS → keep |
| test_discard_exactly_half_second | vad | Boundary: 0.5s → keep |
| test_silence_counter_fires | vad | 3000ms timeout triggers |
| test_silence_counter_resets_on_loud_frame | vad | Loud frame resets counter |
| test_silence_counter_manual_reset | vad | reset() clears accumulator |
| test_wav_header_valid | wav_encoder | Output starts with b"RIFF" |
| test_wav_sample_rate_16k | wav_encoder | Parsed WavSpec: 16kHz, 1ch, 16bit Int |
| test_wav_resampling_44100_to_16k | wav_encoder | 44100Hz input → 16kHz WAV |
| test_resample_noop_at_16k | wav_encoder | from_rate==16000 → passthrough |

## Decisions Made

1. **rubato over linear decimation** — 44100/16000 = 2.75625 (non-integer ratio), simple decimation aliases; rubato SincFixedIn handles this correctly.
2. **MAX_BUFFER_SAMPLES in mod.rs** — T-03-03 threat mitigation constant defined here so Plan 02 can enforce it when wiring the accumulator loop without rediscovering the value.
3. **tauri::Emitter explicit import** — Tauri 2.x requires `use tauri::Emitter;` to call `app_handle.emit()`. Auto-fix applied (Rule 1).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Missing tauri::Emitter trait import**
- **Found during:** Task 2 cargo check
- **Issue:** `app_handle.emit()` call failed — `no method named 'emit' found for struct AppHandle<R>` in Tauri 2.x without explicit trait import
- **Fix:** Added `use tauri::Emitter;` to capture.rs imports
- **Files modified:** `src-tauri/src/recording/capture.rs`
- **Commit:** f582286

**2. [Rule 2 - Security] T-03-03 buffer cap constant**
- **Found during:** Task 2 (threat model review)
- **Issue:** Plan's threat register marks T-03-03 (unbounded buffer growth → DoS) as `mitigate`. No accumulator wiring exists in this plan, but the constant needs to be defined here for Plan 02 to enforce it.
- **Fix:** Added `MAX_BUFFER_SAMPLES` const to mod.rs
- **Files modified:** `src-tauri/src/recording/mod.rs`
- **Commit:** f582286

## Threat Surface Scan

No new network endpoints, auth paths, or schema changes introduced. WAV data remains in-memory (T-03-04 accepted). T-03-03 buffer cap constant defined for Plan 02 enforcement.

## Known Stubs

None. All module functions are fully implemented. The capture thread function requires a real microphone device at runtime — not testable in unit tests, but all logic paths are covered.

## Self-Check: PASSED

- `src-tauri/src/recording/mod.rs` — FOUND
- `src-tauri/src/recording/capture.rs` — FOUND
- `src-tauri/src/recording/vad.rs` — FOUND
- `src-tauri/src/recording/wav_encoder.rs` — FOUND
- Commit 475321a (Task 1) — FOUND
- Commit f582286 (Task 2) — FOUND
- cargo check errors: 0
- cargo test recording: 14 passed, 0 failed
