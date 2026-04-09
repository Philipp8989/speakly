# Technology Stack

**Project:** Speakly — Cross-Platform Voice-to-Text Desktop App
**Researched:** 2026-04-09
**Confidence:** HIGH (core framework), MEDIUM (audio/STT integration)

---

## Recommended Stack

### Desktop Shell

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| Tauri | 2.10.3 | Cross-platform desktop shell | Stable since Oct 2024. Bundle size ~6MB vs Electron ~120MB. Native OS integration. Rust backend gives direct access to system APIs needed for audio, hotkeys, clipboard. macOS + Windows first-class. |
| Rust | 1.77.2+ | Backend logic | Required by Tauri. Handles audio recording, STT dispatch, clipboard write, paste simulation — all syscall-heavy work that benefits from native performance. |
| Vite | 6.x | Frontend build tool | Official Tauri recommendation for SPA frameworks. Fast HMR. Zero config for React/Svelte. |

### Frontend UI

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| React | 19.x | UI framework | Larger ecosystem than Svelte — more Tauri community examples, more Claude/AI UI patterns. Tauri 2 supports both, but React has better TypeScript tooling and the team likely already knows it. VoxPen Desktop (closest open-source reference) uses React + Tauri. |
| TypeScript | 5.x | Type safety | Standard for Tauri frontend. Tauri JS bindings are typed. |
| Tailwind CSS | 4.x | Styling | Minimal CSS, utility-first, good for small app UIs. No runtime overhead in Tauri. |

**Alternative rejected:** Svelte — smaller ecosystem, fewer Tauri examples for this specific use case. Performance gain is negligible for a tray app with minimal DOM.

### Audio Recording (Rust side)

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| cpal | 0.16.0 | Low-level audio input from microphone | The standard cross-platform audio I/O library in Rust. Supports WASAPI (Windows), CoreAudio (macOS). Pure Rust, no C deps. Used internally by `tauri-plugin-mic-recorder` and `tauri-plugin-audio-recorder`. |
| hound | 3.5+ | Encode captured PCM frames to WAV | Standard WAV encoder in Rust. whisper-rs and OpenAI API both accept WAV. Simple API. |

**Implementation choice:** Write a custom Tauri command in Rust using cpal directly rather than using `tauri-plugin-mic-recorder` (v2.0.0, Mar 2025) or `tauri-plugin-audio-recorder`. Reason: the plugins are community-maintained with limited documentation and sparse adoption; direct cpal usage gives full control over the recording loop (start/stop on hotkey, PCM sample rate, channel config). The audio pipeline is core functionality — not a good place to depend on an unmaintained plugin.

**Audio format pipeline:**
```
Microphone → cpal (PCM f32 frames) → hound (WAV encode) → in-memory buffer
→ OpenAI Whisper API (multipart/form-data) or whisper-rs (raw PCM)
```

OpenAI API accepts WAV (max 25MB). whisper-rs accepts f32 PCM directly. Store both paths.

### Speech-to-Text

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| openai crate (or HTTP via reqwest) | — | Cloud STT via Whisper API | Primary engine. Best transcription quality. Supports 50+ languages with auto-detection. New `gpt-4o-transcribe` model available (higher quality than `whisper-1`). Initiated from Rust backend via `reqwest` + multipart form. |
| whisper-rs | 0.16.0 | Local/offline STT via whisper.cpp | Rust bindings for whisper.cpp. Latest version 0.16.0 (2026-03-12). Supports Metal (macOS GPU), CUDA, Vulkan. Offline fallback when API unavailable. Requires shipping model file (~40MB for base, ~140MB for medium). |

**Model recommendation for local:** `ggml-base.en` for English-only builds (fastest, ~40MB), `ggml-base` for multilingual. Expose model choice in settings.

**Cloud model recommendation:** Call `whisper-1` as default (battle-tested), expose `gpt-4o-transcribe` as an opt-in beta option in settings.

**Integration pattern:**
```rust
// Tauri command called after recording stops
#[tauri::command]
async fn transcribe(audio_bytes: Vec<u8>, mode: &str) -> Result<String, String> {
    match mode {
        "cloud" => call_openai_whisper_api(audio_bytes).await,
        "local" => run_whisper_rs(audio_bytes),
        _ => Err("Unknown mode".into())
    }
}
```

### AI Text Reformulation

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| reqwest | 0.12+ | HTTP client for Claude API | Standard async Rust HTTP client. Used for both OpenAI Whisper and Claude API calls. Single dependency covers both. |
| Anthropic Claude API (claude-3-5-haiku) | current | AI commands: rephrase, translate, shorten | Team already uses Claude. Haiku is fast and cheap for short reformulation tasks. Exposed as Tauri commands from Rust. |

### Clipboard + Paste Simulation

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| @tauri-apps/plugin-clipboard-manager | 2.x | Write transcribed text to clipboard | Official Tauri plugin. Cross-platform. Stable. Used from frontend JS or Rust side. |
| enigo | 0.2+ | Simulate Cmd+V / Ctrl+V after clipboard write | Cross-platform keyboard input simulation in Rust. Supports macOS + Windows natively. The correct pattern is: write to clipboard, then enigo fires the platform paste shortcut. Works in any active text field. |

**Paste flow:**
```rust
// Rust backend Tauri command
async fn insert_text(text: String) {
    // 1. Write to clipboard (arboard or tauri plugin)
    clipboard.write_text(&text);
    // 2. Small delay (50ms) to ensure clipboard is set
    tokio::time::sleep(Duration::from_millis(50)).await;
    // 3. Simulate Cmd+V (macOS) or Ctrl+V (Windows)
    let mut enigo = Enigo::new(&Settings::default()).unwrap();
    #[cfg(target_os = "macos")]
    { enigo.key(Key::Meta, Direction::Press); enigo.key(Key::Unicode('v'), Direction::Click); enigo.key(Key::Meta, Direction::Release); }
    #[cfg(target_os = "windows")]
    { enigo.key(Key::Control, Direction::Press); enigo.key(Key::Unicode('v'), Direction::Click); enigo.key(Key::Control, Direction::Release); }
}
```

**Why not OS-level accessibility APIs (CGEventPost, SendInput)?** Enigo wraps these under the hood. No need for raw unsafe bindings.

### Global Hotkey

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| tauri-plugin-global-shortcut | 2.x | Register/unregister global keyboard shortcut | Official Tauri plugin. Supported on macOS + Windows. User-configurable hotkey stored in plugin-store. Uses `CommandOrControl+Shift+Space` as default. |

**Usage pattern (JS side):**
```typescript
import { register, unregister } from '@tauri-apps/plugin-global-shortcut';

await register('CommandOrControl+Shift+Space', () => {
  invoke('toggle_recording');
});
```

**macOS permission note:** macOS requires Accessibility permission for global shortcuts. Must add `NSAppleEventsUsageDescription` to Info.plist and handle the permission prompt on first launch.

### Persistent Settings

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| @tauri-apps/plugin-store | 2.x | Store API keys, hotkey preference, STT mode | Official Tauri plugin. JSON file on disk. Simple key-value API. Intended exactly for app preferences. Auto-saves on write. |

**What to store:**
- `openai_api_key` — user's own key
- `anthropic_api_key` — user's own key
- `stt_mode` — `"cloud"` | `"local"`
- `hotkey` — e.g. `"CommandOrControl+Shift+Space"`
- `language` — `"auto"` | ISO 639-1 code
- `whisper_model` — local model filename

### System Tray

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| Tauri tray-icon (built-in) | 2.10.3 | Status indicator in menu bar / taskbar | Built into Tauri 2.x as `tray-icon` feature. No extra plugin needed. Supports dynamic icon swap (idle / recording / processing). |

**Known issue:** Reported bug (#13770) on macOS where tray icon occasionally doesn't appear after app launch in latest builds. Workaround: use `iconAsTemplate: false` and provide explicit PNG. Monitor Tauri releases for fix.

---

## Alternatives Considered

| Category | Recommended | Alternative | Why Not |
|----------|-------------|-------------|---------|
| Desktop shell | Tauri 2 | Electron | 20x larger bundle, higher RAM usage, no Rust backend for direct syscalls |
| Frontend | React | Svelte | Fewer Tauri+STT community examples, no meaningful perf advantage for tray app |
| Local STT | whisper-rs | rwhisper | whisper-rs is more mature (0.16.0 vs rwhisper experimental), better GPU support |
| Clipboard | plugin-clipboard-manager | arboard (direct) | Official plugin is better supported; arboard usable as Rust-side fallback if needed |
| Input simulation | enigo | rdev / libxdo | enigo is the most maintained cross-platform crate for macOS+Windows; rdev has issues on macOS Sonoma |
| Hotkey | plugin-global-shortcut | global-hotkey crate | Plugin is the Tauri-blessed path, already has JS bindings, simpler integration |
| Settings | plugin-store | SQLite (tauri-plugin-sqlite) | Settings volume is tiny (< 10 keys), SQLite is overkill; plugin-store is exact fit |
| Cloud STT | OpenAI Whisper API | AssemblyAI, Deepgram | Team already using OpenAI; Whisper API has best language coverage; no extra vendor account |
| AI commands | Claude API (Haiku) | GPT-4o-mini | Team already uses Claude; equivalent cost/quality for reformulation tasks |
| HTTP client | reqwest | ureq | reqwest is async, needed for non-blocking Tauri commands; ureq is sync-only |

---

## Installation

```bash
# Create project
cargo install create-tauri-app
cargo create-tauri-app speakly --template react-ts

# Tauri plugins (add to src-tauri/Cargo.toml)
cargo add tauri-plugin-global-shortcut --target 'cfg(any(target_os = "macos", windows))'
cargo add tauri-plugin-clipboard-manager
cargo add tauri-plugin-store
cargo add cpal
cargo add hound
cargo add whisper-rs --features metal  # Metal for macOS GPU
cargo add enigo
cargo add reqwest --features json,multipart
cargo add tokio --features full
cargo add serde --features derive
cargo add serde_json

# Frontend (from project root)
pnpm add @tauri-apps/plugin-global-shortcut
pnpm add @tauri-apps/plugin-clipboard-manager
pnpm add @tauri-apps/plugin-store
```

---

## Confidence Assessment

| Component | Confidence | Source |
|-----------|------------|--------|
| Tauri 2.10.3 as shell | HIGH | Official GitHub releases, verified Mar 2025 |
| plugin-global-shortcut | HIGH | Official Tauri docs, confirmed macOS+Windows |
| plugin-clipboard-manager | HIGH | Official Tauri plugin, documented |
| enigo for paste sim | HIGH | GitHub (active, macOS+Windows confirmed), VoxPen reference |
| cpal for audio capture | HIGH | Most-used Rust audio crate, stable 0.16 |
| whisper-rs for local STT | HIGH | Crates.io 0.16.0 released 2026-03-12, active |
| plugin-store for settings | HIGH | Official Tauri plugin |
| React over Svelte | MEDIUM | Community patterns suggest both work; recommendation based on ecosystem size |
| Tauri tray-icon stability | MEDIUM | Known bug on macOS (#13770), workaround available |
| Local model bundle size | MEDIUM | Based on public whisper.cpp model sizes; needs validation for distribution |

---

## Sources

- [Tauri 2.10.3 Release](https://github.com/tauri-apps/tauri/releases)
- [Tauri 2.0 Stable Release Announcement](https://v2.tauri.app/blog/tauri-20/)
- [Tauri Global Shortcut Plugin](https://v2.tauri.app/plugin/global-shortcut/)
- [Tauri Clipboard Plugin](https://v2.tauri.app/plugin/clipboard/)
- [Tauri Store Plugin](https://v2.tauri.app/plugin/store/)
- [Tauri System Tray Docs](https://v2.tauri.app/learn/system-tray/)
- [whisper-rs 0.16.0 on Docs.rs](https://docs.rs/crate/whisper-rs/latest)
- [cpal on GitHub](https://github.com/RustAudio/cpal)
- [enigo on GitHub](https://github.com/enigo-rs/enigo)
- [VoxPen Desktop (reference implementation)](https://github.com/soanseng/voxpen-desktop)
- [tauri-plugin-global-shortcut on crates.io](https://crates.io/crates/tauri-plugin-global-shortcut)
- [tauri-plugin-audio-recorder on GitHub](https://github.com/brenogonzaga/tauri-plugin-audio-recorder)
