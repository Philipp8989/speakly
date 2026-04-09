# Architecture Patterns: Tauri Voice-to-Text Desktop App

**Domain:** Cross-platform voice-to-text desktop app (Speakly)
**Researched:** 2026-04-09
**Confidence:** HIGH — verified against Tauri v2 official docs + real-world Tauri STT reference apps (Pluely, Pothook)

---

## Recommended Architecture

```
┌────────────────────────────────────────────────────────────────────┐
│  FRONTEND (WebView — React/TypeScript)                             │
│                                                                    │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────────┐ │
│  │  TrayPopup   │  │  Settings    │  │  TranscriptionOverlay    │ │
│  │  (status,    │  │  (hotkey,    │  │  (live status: idle /    │ │
│  │   history)   │  │   API keys,  │  │   recording / processing)│ │
│  │              │  │   engine)    │  │                          │ │
│  └──────┬───────┘  └──────┬───────┘  └──────────┬─────────────┘ │
│         └─────────────────┴──────────────────────┘               │
│                          IPC Bridge (invoke / listen)             │
└────────────────────────────┬──────────────────────────────────────┘
                             │
         ┌───────────────────▼───────────────────────────────┐
         │  RUST BACKEND (Tauri Core)                        │
         │                                                   │
         │  ┌────────────┐  ┌─────────────┐  ┌───────────┐  │
         │  │  Hotkey    │  │   Audio     │  │  Config   │  │
         │  │  Manager   │  │   Capture   │  │  Store    │  │
         │  │ (tauri-    │  │  (cpal +    │  │ (tauri-   │  │
         │  │  plugin-   │  │  platform   │  │  plugin-  │  │
         │  │  global-   │  │  API)       │  │  store)   │  │
         │  │  shortcut) │  └──────┬──────┘  └───────────┘  │
         │  └──────┬─────┘         │                         │
         │         │          WAV buffer                      │
         │         │               │                         │
         │         │        ┌──────▼──────────────────────┐  │
         │         │        │  STT Router                 │  │
         │         │        │  - Whisper API (cloud)      │  │
         │         │        │  - whisper.cpp (local)      │  │
         │         │        └──────┬──────────────────────┘  │
         │         │               │  transcribed text        │
         │         │        ┌──────▼──────────────────────┐  │
         │         │        │  AI Command Handler          │  │
         │         │        │  (Claude API, optional)     │  │
         │         │        └──────┬──────────────────────┘  │
         │         │               │  final text              │
         │         │        ┌──────▼──────────────────────┐  │
         │         └───────►│  Text Injector              │  │
         │                  │  (clipboard write +          │  │
         │                  │   enigo Cmd+V/Ctrl+V sim)   │  │
         │                  └─────────────────────────────┘  │
         └───────────────────────────────────────────────────┘
```

---

## Component Boundaries

### Rust Backend Owns

| Component | Responsibility | Crate(s) |
|-----------|---------------|----------|
| HotkeyManager | Register/deregister global hotkeys; fire trigger events | `tauri-plugin-global-shortcut` |
| AudioCapture | Open mic via platform audio API; accumulate PCM into buffer; emit recording-started/stopped events | `cpal` (cross-platform audio I/O) |
| STTRouter | Choose cloud vs local engine; call Whisper API with WAV bytes OR invoke whisper.cpp binary; return transcript string | `reqwest` (HTTP), `whisper-rs` or subprocess for whisper.cpp |
| AICommandHandler | Send transcript + command to Claude API; return reformulated text | `reqwest` |
| TextInjector | Write text to clipboard; simulate Cmd+V (macOS) or Ctrl+V (Windows) in previously-focused window | `arboard` (clipboard) + `enigo` (key simulation) |
| ConfigStore | Persist settings (hotkey, API keys, engine choice, language) | `tauri-plugin-store` |
| AppState | Thread-safe shared state (recording flag, last active window handle) | `Arc<Mutex<T>>` managed via `.manage()` |

### Frontend Owns

| Component | Responsibility |
|-----------|---------------|
| SystemTrayMenu | Show recording status icon; quick access to settings; toggle on/off |
| StatusOverlay | Floating minimal UI: idle / recording / processing / error indicator |
| SettingsPanel | Form for hotkey binding, API keys, engine selection, language |
| TranscriptionHistory | Optional: last N transcriptions with copy button |
| AppState (React) | Local UI state; settings synced from/to backend via invoke |

### Strict Boundary Rule

The frontend NEVER touches audio, files, OS clipboard, or key simulation directly. It only calls `invoke()` commands and listens to `emit()` events. All side effects live in Rust.

---

## Data Flow

### Primary Flow (recording to injected text)

```
1. User presses hotkey
      │
      ▼
2. HotkeyManager fires (Rust)
   → emits "recording-state-changed" event to frontend (UI: show red dot)
   → opens mic via cpal, starts buffering PCM
      │
      ▼
3. User releases hotkey (or second press)
   → cpal stops capture
   → PCM buffer encoded to WAV in memory
      │
      ▼
4. STTRouter receives WAV bytes
   → if engine=cloud: POST to Whisper API, await JSON response
   → if engine=local: pass WAV path to whisper.cpp subprocess, parse stdout
   → returns transcript: String
      │
      ▼
5. (Optional) AI Command Handler
   → if AI command active: POST transcript + prompt to Claude API
   → returns reformulated text
      │
      ▼
6. TextInjector
   → records currently active window (captured before hotkey triggered audio)
   → writes final text to clipboard via arboard
   → uses enigo to simulate Cmd+V (macOS) / Ctrl+V (Windows)
   → emits "transcription-complete" event to frontend (UI: show brief notification)
      │
      ▼
7. Text appears in previously active window
```

### Settings Flow

```
Frontend (SettingsPanel) 
  → invoke("save_settings", { hotkey, api_key, engine, language })
  → Rust: ConfigStore persists to disk; HotkeyManager re-registers hotkey
  → invoke("get_settings") on startup to hydrate frontend state
```

### Status Event Flow (Backend → Frontend)

```
Rust emits:
  "recording-state-changed"  { recording: bool }
  "transcription-complete"   { text: String, duration_ms: u32 }
  "transcription-error"      { message: String }
  "engine-status"            { engine: "cloud"|"local", available: bool }

Frontend listens via:
  listen("recording-state-changed", handler)
  ...
```

---

## IPC Pattern

Tauri v2 uses two primitives:

**Commands (Frontend → Backend, request/response):**
```typescript
// Frontend
const text = await invoke<string>("start_recording")
const settings = await invoke<Settings>("get_settings")
await invoke("save_settings", { payload: settings })
```
```rust
// Rust
#[tauri::command]
async fn start_recording(state: State<'_, AppState>) -> Result<(), String> { ... }
```

**Events (Backend → Frontend, fire-and-forget):**
```rust
// Rust emits
app_handle.emit("recording-state-changed", RecordingPayload { recording: true }).unwrap();
```
```typescript
// Frontend listens
await listen<RecordingPayload>("recording-state-changed", (event) => {
  setRecording(event.payload.recording)
})
```

Key rule: Commands for user-initiated actions (start, stop, save settings). Events for state changes the backend pushes unprompted (recording status, transcription results, errors).

---

## State Management

### Rust Managed State (thread-safe, initialized in `lib.rs`)

```rust
struct RecordingState {
    is_recording: AtomicBool,
    audio_task: Arc<Mutex<Option<JoinHandle<()>>>>,
    audio_buffer: Arc<Mutex<Vec<f32>>>,
}

struct AppConfig {
    hotkey: Mutex<String>,
    api_key_whisper: Mutex<String>,
    api_key_claude: Mutex<String>,
    engine: Mutex<Engine>,  // Cloud | Local
    language: Mutex<String>,
}
```

All state registered via `.manage()` at startup and injected into commands via `State<'_, T>`.

### Frontend State (React)

Minimal — mirrors what backend emits. Settings fetched once on startup via `invoke("get_settings")`. UI state (is_recording, last_transcript, error) derived from event listeners. No local persistence needed (ConfigStore owns persistence).

---

## Platform-Specific Concerns

| Concern | macOS | Windows |
|---------|-------|---------|
| Mic permission | Must request via OS dialog; handle denial gracefully | Standard permissions dialog |
| Audio API | CoreAudio via cpal | WASAPI via cpal |
| Paste simulation | `enigo`: Cmd+V | `enigo`: Ctrl+V |
| Active window focus | Must capture before hotkey opens recording — hotkey fires, focus still on target app | Same behavior |
| Global hotkey | `tauri-plugin-global-shortcut` handles both | Same |
| Tray icon | PNG required | ICO required (both needed in assets) |
| Accessibility permission | Required for enigo key simulation on macOS (Accessibility API) | Not required |

**Critical:** On macOS, `enigo` requires Accessibility permission for key simulation. The app must prompt for this on first use and handle the case where it's denied (fall back to "text copied to clipboard" notification only).

---

## Suggested Build Order (Phase Dependencies)

```
Phase 1: Foundation
  → Tauri project scaffold + system tray + basic settings UI
  → ConfigStore (hotkey, API keys, engine)
  → No audio yet — validates project runs on both platforms

Phase 2: Recording Core
  → HotkeyManager (global shortcut register/deregister)
  → AudioCapture (cpal mic input, WAV buffer)
  → IPC: start_recording / stop_recording commands
  → Frontend status indicator
  Dependency: Phase 1 (config for hotkey binding)

Phase 3: Cloud Transcription
  → STTRouter cloud path (Whisper API via reqwest)
  → TextInjector (clipboard + enigo paste simulation)
  → End-to-end flow: hotkey → audio → Whisper API → injected text
  Dependency: Phase 2 (audio buffer)

Phase 4: Local Transcription
  → STTRouter local path (whisper.cpp integration)
  → Model download/management UI
  → Engine selection toggle in settings
  Dependency: Phase 3 (STTRouter abstraction already exists)

Phase 5: AI Commands
  → AICommandHandler (Claude API)
  → Command selection UI (quick action menu)
  Dependency: Phase 3 (transcript already produced)

Phase 6: Polish
  → Error handling, permission prompts (macOS Accessibility)
  → Transcription history
  → Platform-specific packaging (DMG for macOS, MSI/NSIS for Windows)
```

Each phase delivers a runnable, testable increment. Phases 1-3 constitute the MVP.

---

## Anti-Patterns to Avoid

### Anti-Pattern 1: Audio capture in frontend via getUserMedia
**Why bad:** Adds ~300-400ms latency vs native audio API; browser sandbox limits; cross-platform inconsistency.
**Instead:** Use cpal in Rust backend for native CoreAudio/WASAPI access.

### Anti-Pattern 2: Text injection via OS accessibility APIs (AT-SPI, AX)
**Why bad:** Fragile, app-specific, requires elevated permissions, unreliable across apps.
**Instead:** Clipboard write + paste simulation via enigo is the most reliable cross-platform method.

### Anti-Pattern 3: Storing audio on disk between phases
**Why bad:** Unnecessary I/O, privacy concern, temp file cleanup complexity.
**Instead:** Keep audio as in-memory Vec<f32> / WAV bytes; only write to disk if whisper.cpp requires a file path.

### Anti-Pattern 4: Frontend managing recording state
**Why bad:** Frontend can be hidden/minimized; hotkey fires regardless of UI visibility.
**Instead:** RecordingState lives in Rust AppState; frontend is a passive display layer.

### Anti-Pattern 5: Single monolithic Tauri command for full pipeline
**Why bad:** Blocks UI thread, no progress feedback, harder to test individual steps.
**Instead:** Commands per stage (start_recording, stop_and_transcribe, apply_ai_command, inject_text) with events for progress.

---

## Sources

- [Tauri v2 Architecture](https://v2.tauri.app/concept/architecture/) — HIGH confidence
- [Tauri IPC: Calling Rust from Frontend](https://v2.tauri.app/develop/calling-rust/) — HIGH confidence
- [Tauri IPC: Calling Frontend from Rust](https://v2.tauri.app/develop/calling-frontend/) — HIGH confidence
- [Tauri Global Shortcut Plugin](https://v2.tauri.app/plugin/global-shortcut/) — HIGH confidence
- [Tauri System Tray](https://v2.tauri.app/learn/system-tray/) — HIGH confidence
- [Tauri Clipboard Plugin](https://v2.tauri.app/plugin/clipboard/) — HIGH confidence
- [Pluely Backend Architecture (DeepWiki)](https://deepwiki.com/iamsrikanthnani/pluely/7-backend-core-(taurirust)) — MEDIUM confidence (third-party analysis)
- [enigo — cross-platform input simulation in Rust](https://github.com/enigo-rs/enigo) — HIGH confidence
- [tauri-plugin-mic-recorder](https://crates.io/crates/tauri-plugin-mic-recorder) — MEDIUM confidence
- [Pothook — Tauri + Whisper.cpp app](https://github.com/acknak/pothook) — MEDIUM confidence (reference implementation)
