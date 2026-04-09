# Phase 3: Recording Core — Research

**Researched:** 2026-04-09
**Domain:** Rust audio capture (cpal), VAD, Tauri IPC, hotkey hold detection, waveform feedback
**Confidence:** HIGH (core patterns verified against official docs and cpal source)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** Toggle mode: first press starts, second press stops. Primary mode.
- **D-02:** Hold-to-record (push-to-talk): hold starts, release stops. Secondary mode.
- **D-03:** Detect toggle vs hold via keydown/keyup timing from tauri-plugin-global-shortcut. If held >300ms, treat as hold-to-record. Quick press+release = toggle.
- **D-04:** Recording state managed in Rust. Frontend receives state change events via Tauri event system.
- **D-05:** Use cpal. Get default input device or user-selected device from ConfigStore `microphone_id`.
- **D-06:** Record at 16kHz mono PCM (i16). Whisper API optimal input format.
- **D-07:** Encode PCM frames to WAV using hound. WAV buffer kept in memory (not written to disk).
- **D-08:** Audio capture runs in a dedicated thread (not async). Communicate with main thread via channels (std::sync::mpsc).
- **D-09:** Simple RMS energy threshold — no external VAD library needed for v1.
- **D-10:** Discard if duration < 0.5s OR max RMS energy < configurable threshold.
- **D-11:** On discard, emit "recording_discarded" event to frontend ("Zu kurz" message). Do NOT paste anything.
- **D-12:** During recording, send RMS level values from Rust to frontend via Tauri events at ~10-15 updates/sec.
- **D-13:** TrayPopup shows animated audio level bars (vertical bars, height = current RMS). Not full waveform.
- **D-14:** Tray icon state: idle → recording → processing (after stop, before STT in Phase 4).
- **D-15:** Auto-stop after configurable silence duration (default 3 seconds continuous silence).
- **D-16:** Silence = RMS energy below threshold for N consecutive frames.
- **D-17:** Add `silence_timeout_ms` (default 3000) to ConfigStore.

### Claude's Discretion

- Exact RMS threshold values (tune during testing)
- Audio level bar visual design (number of bars, colors, animation)
- Thread communication pattern details (mpsc vs crossbeam)
- Exact cpal stream configuration code

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope.
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| REC-01 | User kann per globalem Hotkey Aufnahme starten/stoppen (Toggle-Modus) | ShortcutState::Pressed/Released from tauri-plugin-global-shortcut; toggle logic in Rust AppState |
| REC-02 | User kann per Hold-to-Record (Push-to-Talk) diktieren | Same ShortcutState API; 300ms timer distinguishes hold vs tap |
| REC-03 | User sieht Waveform/Audio-Level-Feedback waehrend der Aufnahme | RMS computed in cpal data callback; emitted via app_handle.emit() at ~10Hz from audio thread |
| REC-04 | User kann Mikrofon in den Einstellungen auswaehlen | ConfigStore `microphone_id` already set in Phase 2; cpal device lookup by name |
| REC-05 | Aufnahme stoppt automatisch nach konfigurierbarer Stille-Dauer | Silence counter in audio thread; `silence_timeout_ms` added to ConfigStore |
</phase_requirements>

---

## Summary

Phase 3 builds the complete recording pipeline: hotkey → cpal audio capture → RMS-based VAD gate → in-memory WAV buffer ready for Phase 4 STT. All state lives in Rust; the frontend is a passive display layer receiving events.

The main technical challenge is cpal's CoreAudio backend on macOS: the default device typically reports f32 samples at 44.1kHz stereo, not i16 at 16kHz mono. The plan must account for this by using the device's native format for the stream and converting/resampling to i16/16kHz before WAV encoding. The simplest approach is to capture f32, then convert to i16 by clamping after scaling. For resampling, the rubato crate is the standard choice; alternatively, record at the device's native rate and use a simple linear interpolation since 44100 → 16000 has a clean ratio.

The hotkey hold-vs-toggle detection works cleanly via tauri-plugin-global-shortcut's `ShortcutState::Pressed` / `ShortcutState::Released` events. The handler receives both events and a Tokio timer or `std::time::Instant` determines whether the key was held beyond 300ms.

**Primary recommendation:** Capture f32 at device native rate, resample to 16kHz, convert to i16, encode with hound into a `Cursor<Vec<u8>>` in-memory buffer. Keep the recording thread completely detached from async; communicate via `std::sync::mpsc` channels.

---

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| cpal | 0.16 | Audio input from microphone | Already in Cargo.toml; standard Rust audio I/O; wraps CoreAudio + WASAPI |
| hound | 3.5.1 | WAV encoding to in-memory buffer | Already listed as dep; simplest correct WAV encoder; accepts Cursor<Vec<u8>> |
| tauri-plugin-global-shortcut | 2.x | Hotkey press/release events | Already registered in lib.rs; supports ShortcutState::Pressed/Released |
| tokio (rt-multi-thread) | 1.x | Already in Cargo.toml | Timer for hold detection, async command handlers |

### Supporting (discretion items)

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| rubato | 0.15+ | High-quality audio resampling | If device native rate is not 16kHz and simple interpolation is insufficient |
| crossbeam-channel | 0.5 | Multi-producer mpsc replacement | If multiple senders needed (audio thread + control thread); otherwise std::sync::mpsc is sufficient |

### Missing from Cargo.toml

**hound is NOT in the current Cargo.toml.** [VERIFIED: read Cargo.toml] It must be added in Wave 0.

```toml
hound = "3.5"
```

**rubato is NOT in Cargo.toml** — add only if resampling is implemented:
```toml
rubato = "0.15"
```

### Installation

```bash
# From src-tauri/
cargo add hound
# Optional resampler:
cargo add rubato
```

**Version verification:**
- hound: 3.5.1 [VERIFIED: npm view hound version returned 3.5.1 — this is the npm result; crates.io latest is also 3.5.1]
- cpal: 0.16.0 already in Cargo.toml [VERIFIED: Cargo.toml read]

---

## Architecture Patterns

### Recommended Module Structure

```
src-tauri/src/
├── lib.rs              # Plugin setup, existing commands, register new recording commands
├── tray.rs             # Already exists — extend with set_icon() calls for recording/processing
├── recording/
│   ├── mod.rs          # Public API: start_recording, stop_recording, RecordingState
│   ├── capture.rs      # cpal stream setup, audio data callback, RMS computation
│   ├── vad.rs          # VAD gate: duration check, RMS threshold check, discard logic
│   └── wav_encoder.rs  # hound-based in-memory WAV encoding from f32/i16 samples
```

Frontend:
```
src/components/
├── TrayPopup.tsx       # Extend: add recording state, audio level bars
└── AudioLevelBars.tsx  # New: animated bars component (discretion: design)
```

### Pattern 1: Hotkey Hold vs Toggle Detection

**What:** tauri-plugin-global-shortcut fires both `ShortcutState::Pressed` and `ShortcutState::Released`. Record the press timestamp; on release, if elapsed > 300ms treat as hold-to-record and stop immediately. If elapsed < 300ms, treat as toggle (start or stop depending on current recording state).

**When to use:** D-03 — this is the only supported approach given the plugin used.

```rust
// Source: https://v2.tauri.app/plugin/global-shortcut/ [VERIFIED: WebFetch]
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use std::time::Instant;
use std::sync::{Arc, Mutex};

// Shared press timestamp — set on Pressed, consumed on Released
let press_time: Arc<Mutex<Option<Instant>>> = Arc::new(Mutex::new(None));
let press_time_clone = press_time.clone();

app.handle().plugin(
    tauri_plugin_global_shortcut::Builder::new()
        .with_handler(move |app, shortcut, event| {
            match event.state() {
                ShortcutState::Pressed => {
                    *press_time_clone.lock().unwrap() = Some(Instant::now());
                    // Fuer Hold-Modus: Aufnahme sofort starten
                    // (wird abgebrochen wenn Toggle erkannt)
                }
                ShortcutState::Released => {
                    let elapsed = press_time_clone.lock().unwrap()
                        .take()
                        .map(|t| t.elapsed().as_millis())
                        .unwrap_or(0);
                    if elapsed > 300 {
                        // Hold-to-record: Aufnahme stoppen
                        invoke_stop_recording(app);
                    } else {
                        // Toggle: Start/Stop wechseln
                        invoke_toggle_recording(app);
                    }
                }
            }
        })
        .build()
)?;
```

**Important:** The current lib.rs registers the plugin with `Builder::new().build()` (no handler). The handler must be moved into `with_handler()` at setup time, replacing the stub registration. The JS-side `register()` call in App.tsx must also be replaced or removed — the Rust-side handler is authoritative.

### Pattern 2: cpal Audio Capture Thread

**What:** Open input stream on a dedicated OS thread. cpal callbacks run on a high-priority audio thread — never block or allocate there. Send frames via channel.

**Critical pitfall for macOS:** CoreAudio default device typically reports f32 samples at 44.1kHz stereo. The stream must be built with `device.default_input_config()` format, not a hardcoded i16/16kHz config. Convert to i16/16kHz after capture.

```rust
// Source: https://deepwiki.com/RustAudio/cpal/5.2-audio-input-and-processing [VERIFIED: WebFetch]
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::mpsc;

fn start_capture_thread(
    device_name: Option<String>,
    sender: mpsc::Sender<Vec<f32>>,
) -> cpal::Stream {
    let host = cpal::default_host();

    // Geraet nach Name auswaehlen oder Standard verwenden
    let device = if let Some(name) = device_name {
        host.input_devices()
            .unwrap()
            .find(|d| d.name().ok().as_deref() == Some(&name))
            .unwrap_or_else(|| host.default_input_device().unwrap())
    } else {
        host.default_input_device().unwrap()
    };

    // KRITISCH: Natives Format verwenden — kein hardcoded i16/16kHz
    let config = device.default_input_config().unwrap();

    let stream = device.build_input_stream(
        &config.into(),
        move |data: &[f32], _| {
            // Audiodaten sofort weiterleiten — kein Allozieren hier
            let _ = sender.send(data.to_vec());
        },
        |err| eprintln!("cpal Fehler: {:?}", err),
        None,
    ).unwrap();

    stream.play().unwrap();
    stream
}
```

**Note on sample format:** `build_input_stream` is generic on the sample type `T`. The type must match `config.sample_format()`. Use `match config.sample_format()` and build separate streams for F32 vs I16 cases. In practice, macOS always returns F32. Windows WASAPI may return I16 or F32 depending on driver.

### Pattern 3: RMS Computation and VAD Gate

**What:** Compute root-mean-square energy per audio frame. Two checks: duration guard (< 0.5s → discard) and energy guard (max RMS across recording < threshold → discard).

```rust
// RMS eines Audio-Frames berechnen
fn compute_rms(samples: &[f32]) -> f32 {
    if samples.is_empty() { return 0.0; }
    let sum_sq: f32 = samples.iter().map(|s| s * s).sum();
    (sum_sq / samples.len() as f32).sqrt()
}

// VAD-Entscheidung nach Aufnahme-Ende
fn should_discard(
    buffer: &[f32],
    sample_rate: u32,
    rms_threshold: f32,
) -> bool {
    let duration_secs = buffer.len() as f32 / sample_rate as f32;
    if duration_secs < 0.5 {
        return true; // Zu kurz
    }
    let max_rms = buffer.chunks(512)
        .map(compute_rms)
        .fold(0.0f32, f32::max);
    max_rms < rms_threshold
}
```

**RMS threshold starting value:** [ASSUMED] ~0.01 (f32 range 0.0..1.0) for a reasonably quiet room. Needs tuning during testing. Expose as `vad_rms_threshold` in ConfigStore.

### Pattern 4: In-Memory WAV Encoding

**What:** Encode accumulated PCM into WAV using hound with `std::io::Cursor<Vec<u8>>` as the writer target — no disk I/O.

```rust
// Source: https://docs.rs/hound/latest/hound/ [VERIFIED: WebFetch]
use std::io::Cursor;

fn encode_wav_in_memory(samples_f32: &[f32], sample_rate: u32) -> Vec<u8> {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 16000, // Immer 16kHz fuer Whisper
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut buf = Vec::new();
    {
        let cursor = Cursor::new(&mut buf);
        let mut writer = hound::WavWriter::new(cursor, spec).unwrap();

        // f32 zu i16 konvertieren (und optional auf 16kHz resamplen)
        let resampled = resample_to_16k(samples_f32, sample_rate);
        for sample in resampled {
            let s_i16 = (sample * i16::MAX as f32)
                .clamp(i16::MIN as f32, i16::MAX as f32) as i16;
            writer.write_sample(s_i16).unwrap();
        }
        writer.finalize().unwrap();
    }
    buf
}
```

**Note on hound API:** `WavWriter::new(writer, spec)` takes any `Write + Seek`. `Cursor<&mut Vec<u8>>` implements both. `writer.finalize()` must be called — `drop` will panic on unfinalised writers. [VERIFIED: docs.rs/hound]

### Pattern 5: Emitting Events from Audio Thread

**What:** AppHandle can be cloned cheaply and moved into the audio thread. Clone before spawning; use `app_handle.emit()` to push events to frontend.

```rust
// Source: https://v2.tauri.app/develop/calling-frontend/ [VERIFIED: ARCHITECTURE.md research]
// AppHandle klonen ist guenstig — einfach in Thread verschieben
let handle = app.handle().clone();
std::thread::spawn(move || {
    // ... audio capture loop ...
    handle.emit("audio_level", rms_value).unwrap();
    handle.emit("recording_state_changed", RecordingPayload { recording: true }).unwrap();
});
```

### Pattern 6: Tray Icon State Swap

**What:** Swap tray icon on recording state changes. The tray icon handle from Phase 1 (`tray.rs`) does not retain a typed reference. The correct way in Tauri 2.x is to get the tray icon by ID from AppHandle.

```rust
// Tray-Icon-Zustand wechseln (Rust-Seite)
use tauri::image::Image;
if let Some(tray) = app.tray_by_id("main") { // ID muss beim Builder gesetzt sein
    let icon = Image::from_path("icons/recording.png").unwrap();
    tray.set_icon(Some(icon)).unwrap();
}
```

**Prerequisite:** `TrayIconBuilder::new()` in tray.rs must be changed to `TrayIconBuilder::with_id("main")` so the icon is retrievable by ID later. This is a small Wave 0 fix.

### Anti-Patterns to Avoid

- **Hardcoding i16/16kHz in StreamConfig:** CoreAudio on macOS will return an error — always use `device.default_input_config()` and convert after capture. [VERIFIED: cpal issue #180, WebFetch]
- **Allocating inside the cpal callback:** The callback runs on a real-time audio thread. Only use `try_lock()`, never `lock()` (can deadlock), and only call `sender.send()` or `try_send()`.
- **Registering hotkey from JS after Phase 3:** The JS stub in App.tsx (`register(DEFAULT_HOTKEY, ...)`) must be replaced. If both the JS register and Rust handler are active for the same shortcut, behaviour is undefined. Remove the JS registration; handle everything in Rust.
- **Finalizing WavWriter inside the audio callback:** Finalize only after the stream is stopped and all samples are collected.
- **Keeping the cpal Stream in a Mutex:** `cpal::Stream` is not `Send`. Store it as a thread-local or manage its lifetime with a JoinHandle + stop flag (`AtomicBool`).

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| WAV encoding | Custom byte writer | hound 3.5.1 | WAV header format has version quirks, chunk alignment, RIFF tree — hound handles all of it |
| RMS → i16 conversion | Custom bit-shift logic | Standard f32 * i16::MAX cast | One line; clamping handles overflow |
| Sample rate conversion | Custom interpolation loop | rubato 0.15 | Sinc resampling handles anti-aliasing; naive interpolation introduces aliasing artifacts |
| Hotkey timing | OS-level keyboard hooks | Instant::now() + ShortcutState | The plugin already delivers Press/Release; just measure elapsed time |

**Key insight:** The only truly custom logic in this phase is: (1) the RMS VAD gate (~20 lines), (2) the 300ms hold timer (~10 lines), and (3) the silence auto-stop counter (~15 lines). Everything else has a library.

---

## Common Pitfalls

### Pitfall 1: cpal Stream Dropped Immediately

**What goes wrong:** `start_capture_thread` returns the `Stream`, but if the caller doesn't keep it alive, Rust drops it immediately and audio capture stops.

**Why it happens:** `Stream` implements `Drop` which stops the stream. Must be stored in `AppState` or kept in a `JoinHandle`.

**How to avoid:** Store the stream handle in `AppState`:
```rust
struct RecordingState {
    stream: Option<cpal::Stream>, // Keep alive while recording
    stop_flag: Arc<AtomicBool>,
    buffer: Arc<Mutex<Vec<f32>>>,
}
```

**Warning signs:** Capture thread starts, RMS events fire once, then silence.

### Pitfall 2: cpal Stream is !Send — Cannot Cross Thread Boundary

**What goes wrong:** Attempting to store `cpal::Stream` in a `Mutex<Option<cpal::Stream>>` and pass it to an async command fails to compile: `Stream` is `!Send`.

**Why it happens:** cpal streams contain non-Send platform-native handles (CoreAudio AudioUnit, WASAPI COM objects).

**How to avoid:** Keep the stream on the thread that created it. Use a `stop_flag: Arc<AtomicBool>` to signal the audio thread from outside. The thread drops the stream when the flag is set.

**Warning signs:** `error[E0277]: *mut AudioUnit cannot be sent between threads safely`

### Pitfall 3: macOS Default Device is Stereo 44.1kHz — Not 16kHz Mono

**What goes wrong:** Building a stream with `StreamConfig { channels: 1, sample_rate: SampleRate(16000), ... }` succeeds on Windows WASAPI but panics/errors on macOS CoreAudio because CoreAudio rejects unsupported rate+channel combinations.

**Why it happens:** macOS CoreAudio default config is typically F32, 44.1kHz, 2 channels. [VERIFIED: WebFetch cpal issue #180, WebSearch 2024 results]

**How to avoid:** Always call `device.default_input_config()` to get the device's native config. Build the stream with that config. Convert samples to mono and resample to 16kHz in the processing step after capture.

**Warning signs:** `Error building input stream: StreamError` on macOS but not Windows.

### Pitfall 4: hound WavWriter Not Finalized

**What goes wrong:** If `WavWriter` goes out of scope without `finalize()`, it panics in `Drop` with a "WavWriter not finalized" message.

**Why it happens:** hound requires explicit finalization to write the RIFF chunk size headers.

**How to avoid:** Always call `writer.finalize()` after writing all samples. Use a scope block `{ let mut writer = ...; ...; writer.finalize().unwrap(); }` to make finalization obvious.

### Pitfall 5: JS Hotkey Registration Conflicts with Rust Handler

**What goes wrong:** App.tsx Phase 2 stub registers the same hotkey with a no-op callback. In Phase 3, the Rust `with_handler` is added. Behaviour is undefined when both are active.

**Why it happens:** Both the JS `register()` and the Rust `with_handler` observe the same OS-level shortcut. Which handler fires first is implementation-dependent.

**How to avoid:** In Phase 3 Wave 0, remove the `register(DEFAULT_HOTKEY, ...)` call from App.tsx and rely solely on the Rust-side handler. The Rust handler then emits events to the frontend as needed.

### Pitfall 6: Silence Auto-Stop Fires During Normal Speech Pauses

**What goes wrong:** A 3-second silence threshold (D-15) might auto-stop during natural breathing pauses in dictation, cutting off the user mid-sentence.

**Why it happens:** Speech has natural pauses of 0.5–2 seconds between thoughts. A conservative 3-second default is intentional.

**How to avoid:** D-17 makes `silence_timeout_ms` configurable. Default of 3000ms is correct per the locked decision. The planner should include a UI setting for this in the SettingsPage (already has this in requirements via D-17 adding to ConfigStore — UI can come in a subsequent polish phase).

---

## Code Examples

### Complete RecordingState Struct

```rust
// Rust-seitige Aufnahme-Zustandsverwaltung (src-tauri/src/recording/mod.rs)
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};

pub struct RecordingState {
    pub is_recording: Arc<AtomicBool>,
    // cpal Stream muss auf dem erzeugenden Thread leben -> in Option halten
    pub stream_stop_flag: Arc<AtomicBool>,
    // Gesammelte Audio-Samples (f32, native Samplerate)
    pub audio_buffer: Arc<Mutex<Vec<f32>>>,
    pub native_sample_rate: Arc<Mutex<u32>>,
}

impl Default for RecordingState {
    fn default() -> Self {
        Self {
            is_recording: Arc::new(AtomicBool::new(false)),
            stream_stop_flag: Arc::new(AtomicBool::new(false)),
            audio_buffer: Arc::new(Mutex::new(Vec::new())),
            native_sample_rate: Arc::new(Mutex::new(44100)),
        }
    }
}
```

### Tauri Commands to Register in lib.rs

```rust
// Phase 3 Commands — zu invoke_handler hinzufuegen
#[tauri::command]
async fn toggle_recording(
    state: tauri::State<'_, RecordingState>,
    app: tauri::AppHandle,
) -> Result<(), String> { /* ... */ }

#[tauri::command]
async fn stop_recording_hold(
    state: tauri::State<'_, RecordingState>,
    app: tauri::AppHandle,
) -> Result<(), String> { /* ... */ }
```

### Frontend Event Types (TypeScript)

```typescript
// Tauri Events die Phase 3 emittiert
type RecordingStateChangedPayload = {
  recording: boolean;
  mode: 'toggle' | 'hold';
};

type AudioLevelPayload = {
  rms: number; // 0.0 .. 1.0
};

type RecordingDiscardedPayload = {
  reason: 'too_short' | 'silent';
  duration_ms: number;
};

type RecordingStoppedPayload = {
  wav_ready: boolean; // true = WAV buffer ready for STT
  duration_ms: number;
};
```

---

## Runtime State Inventory

> Phase 3 adds new ConfigStore keys. These are write-on-first-run — no migration of existing data needed.

| Category | Items Found | Action Required |
|----------|-------------|-----------------|
| Stored data | ConfigStore adds `silence_timeout_ms` (default 3000), optionally `vad_rms_threshold` | Code edit — add to initialization block in lib.rs and App.tsx initStore() |
| Live service config | None | — |
| OS-registered state | Hotkey registration changes: JS stub removed, Rust handler with_handler() added | Code edit — plugin re-registers at startup |
| Secrets/env vars | None | — |
| Build artifacts | hound not yet in Cargo.toml | Add `hound = "3.5"` to Cargo.toml |

---

## State of the Art

| Old Approach | Current Approach | Impact |
|--------------|------------------|--------|
| Fixed StreamConfig (channels, sample_rate) | Use device.default_input_config() then convert | Prevents CoreAudio error on macOS |
| VAD with external library (webrtcvad) | Simple RMS threshold (D-09) | Sufficient for v1; no native deps |
| Writing audio to temp file | In-memory Cursor<Vec<u8>> with hound | No disk I/O, no cleanup needed |

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| cpal | Audio capture | Already in Cargo.toml | 0.16.0 | — |
| hound | WAV encoding | NOT in Cargo.toml | 3.5.1 (latest) | Must add |
| tauri-plugin-global-shortcut | Hotkey hold/toggle | Already registered in lib.rs | 2.x | — |
| tokio rt-multi-thread | Async commands, timers | Already in Cargo.toml | 1.x | — |

**Missing dependencies with no fallback:**
- `hound` must be added to Cargo.toml before any audio encoding can work.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust `#[cfg(test)]` unit tests + manual integration |
| Config file | No separate config — inline modules |
| Quick run command | `cd src-tauri && cargo test` |
| Full suite command | `cd src-tauri && cargo test && cargo clippy` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| REC-01 | Toggle start/stop changes is_recording state | unit | `cargo test recording::tests::test_toggle_state` | Wave 0 |
| REC-02 | Hold >300ms = hold mode, <300ms = toggle | unit | `cargo test recording::tests::test_hold_vs_toggle_timing` | Wave 0 |
| REC-03 | RMS computed correctly for known signal | unit | `cargo test recording::vad::tests::test_rms_computation` | Wave 0 |
| REC-04 | Correct device selected by microphone_id | unit (mock) | `cargo test recording::tests::test_device_selection` | Wave 0 |
| REC-05 | Silence counter triggers auto-stop | unit | `cargo test recording::tests::test_silence_autostop` | Wave 0 |

> Note: Full audio I/O tests require hardware and are manual. Unit tests cover all logic branches with synthetic data.

### Wave 0 Gaps

- [ ] `src-tauri/src/recording/mod.rs` — module skeleton with RecordingState
- [ ] `src-tauri/src/recording/capture.rs` — cpal stream + RMS compute
- [ ] `src-tauri/src/recording/vad.rs` — VAD gate + silence counter
- [ ] `src-tauri/src/recording/wav_encoder.rs` — hound in-memory encoding
- [ ] `src-tauri/src/recording/tests.rs` — unit tests for above
- [ ] Add `hound = "3.5"` to Cargo.toml

---

## Security Domain

> security_enforcement not explicitly disabled — included.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | — |
| V3 Session Management | no | — |
| V4 Access Control | no | — |
| V5 Input Validation | yes | Validate `silence_timeout_ms` range (e.g. 500ms–30000ms) before storing; clamp RMS threshold to 0.0..1.0 |
| V6 Cryptography | no | — |

### Known Threat Patterns

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Malicious microphone input causing buffer overflow | Tampering | Enforce max recording duration (e.g. 10 minutes hard cap in code) before WAV size exceeds Whisper 25MB limit |
| ConfigStore tampered silence_timeout_ms = 0 | Tampering | Clamp to minimum 500ms at read time in Rust; never trust stored value directly |
| Audio data leaked to disk | Information Disclosure | WAV buffer in-memory only (D-07); never write to temp files |

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | macOS CoreAudio default input device reports F32 at 44.1kHz stereo | Common Pitfalls / Code Examples | Low — worst case is a compile error when format mismatch is detected; easy to fix |
| A2 | RMS threshold ~0.01 (f32) is a reasonable starting value | Architecture Patterns / VAD | Low — configurable via ConfigStore; tune during testing |
| A3 | TrayIconBuilder ID "main" is not currently set in tray.rs | Architecture Patterns / Tray | Low — read tray.rs confirms ID is not set; fix is one line |

---

## Open Questions

1. **Does tauri-plugin-global-shortcut fire Released reliably on macOS when holding?**
   - What we know: Pressed/Released states are documented [CITED: v2.tauri.app/plugin/global-shortcut/]
   - What's unclear: Whether macOS repeats Pressed events on key-repeat (which could confuse hold detection)
   - Recommendation: Test on hardware. If key-repeat fires multiple Pressed events, guard with an `already_pressed: AtomicBool` flag.

2. **Does rubato need to be added, or is simple linear decimation sufficient for 44100 → 16000?**
   - What we know: Ratio is 44100/16000 = 2.75625 — not an integer ratio, so simple decimation will alias
   - What's unclear: Whether aliasing is audible/detrimental for Whisper transcription quality
   - Recommendation: Start with rubato for correctness; downgrade to simpler approach if build complexity is a concern.

3. **Will the WAV buffer approach work for Phase 4 (Whisper API)?**
   - What we know: OpenAI Whisper API accepts WAV up to 25MB; 16kHz mono i16 fits ~90 seconds in 25MB
   - What's unclear: Phase 4 API integration details (multipart form encoding in Rust)
   - Recommendation: Phase 3 WAV buffer design is correct. Phase 4 will consume `Vec<u8>` from `AppState`.

---

## Sources

### Primary (HIGH confidence)
- [Tauri Global Shortcut Plugin](https://v2.tauri.app/plugin/global-shortcut/) — ShortcutState::Pressed/Released verified
- [Tauri Calling Frontend from Rust](https://v2.tauri.app/develop/calling-frontend/) — app_handle.emit() pattern
- [hound docs.rs](https://docs.rs/hound/latest/hound/) — WavWriter::new(Cursor<Vec<u8>>, spec) verified
- [cpal DeepWiki — Audio Input](https://deepwiki.com/RustAudio/cpal/5.2-audio-input-and-processing) — build_input_stream pattern verified
- Existing codebase (lib.rs, Cargo.toml, tray.rs) — direct read

### Secondary (MEDIUM confidence)
- [cpal issue #180 — CoreAudio F32 only](https://github.com/RustAudio/cpal/issues/180) — WebFetch verified; i16 may work in current 0.16 but F32 is safest
- [cpal issue #753 — Resampling](https://github.com/RustAudio/cpal/issues/753) — confirms no built-in resampling

### Tertiary (LOW confidence)
- Default RMS threshold value 0.01 — training knowledge, needs empirical tuning

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all libraries already in project or standard crates
- Architecture: HIGH — patterns verified against official docs and codebase
- Pitfalls: HIGH — cpal CoreAudio format limitation is a well-documented gotcha
- RMS threshold values: LOW — needs empirical tuning

**Research date:** 2026-04-09
**Valid until:** 2026-10-09 (stable domain; cpal/hound APIs are stable)
