# Research Summary: Speakly

**Synthesized:** 2026-04-09
**Sources:** STACK.md, FEATURES.md, ARCHITECTURE.md, PITFALLS.md

---

## Executive Summary

Speakly is a cross-platform voice-to-text tray app built on Tauri 2 + Rust + React. The domain is well-understood: multiple open-source reference implementations exist (VoiceInk, Pothook, VoxPen), and the core tech stack (cpal + Whisper API + enigo + tauri-plugin-global-shortcut) has been validated in production. The recommended MVP scope is tight: global hotkey → mic capture → Whisper API → clipboard paste. Phases 1-3 deliver a usable, shippable product. The biggest risks are not architectural but operational: macOS permission handling, clipboard clobber, and Whisper hallucinations on silence must be addressed from day one — they are correctness issues, not polish. Offline (whisper.cpp) and AI post-processing (Claude API) are high-value differentiators but should ship in v1.1 after the core loop is stable.

---

## Recommended Stack

| Layer | Technology | Version | Rationale |
|-------|-----------|---------|-----------|
| Desktop shell | Tauri | 2.10.3 | 6MB bundle, native Rust backend, macOS + Windows first-class |
| Backend language | Rust | 1.77.2+ | Required by Tauri; native audio, clipboard, key simulation |
| Frontend | React + TypeScript | 19.x / 5.x | Larger Tauri ecosystem, better tooling vs Svelte for this use case |
| Styling | Tailwind CSS | 4.x | Utility-first, zero runtime overhead in WebView |
| Build tool | Vite | 6.x | Official Tauri recommendation |
| Audio capture | cpal | 0.16.0 | Cross-platform CoreAudio (macOS) + WASAPI (Windows), pure Rust |
| WAV encoding | hound | 3.5+ | Standard WAV encoder; Whisper API and whisper-rs both accept WAV |
| Cloud STT | OpenAI Whisper API (whisper-1 / gpt-4o-transcribe) | current | Best language coverage, team already on OpenAI |
| Local STT | whisper-rs | 0.16.0 | Rust bindings for whisper.cpp, Metal GPU on macOS, offline fallback |
| AI reformulation | Claude API (claude-3-5-haiku) | current | Team already uses Claude; fast + cheap for short reformulations |
| HTTP client | reqwest | 0.12+ | Async, covers both Whisper and Claude API calls |
| Clipboard | tauri-plugin-clipboard-manager | 2.x | Official Tauri plugin, cross-platform |
| Paste simulation | enigo | 0.2+ | Most maintained cross-platform crate for macOS + Windows |
| Global hotkey | tauri-plugin-global-shortcut | 2.x | Official plugin, Tauri-blessed path |
| Settings persistence | tauri-plugin-store | 2.x | JSON key-value, exact fit for <10 settings |

---

## Table Stakes Features

Must ship in MVP — missing any of these makes the app feel incomplete or unusable:

- Global hotkey to start/stop recording (toggle + hold-to-record, user-configurable)
- Audio level / waveform feedback while recording (waveform or animated bars — spinner is not enough)
- Auto-paste into active text field (clipboard + Cmd+V / Ctrl+V simulation)
- Automatic punctuation (Whisper API handles natively; local requires prompt engineering)
- 50+ language support with auto-detection (Whisper natively supports 99 languages)
- System tray icon with 3 states: idle / recording / processing
- First-run onboarding: hotkey configuration + API key setup
- Settings: hotkey, microphone selection, language, API key management
- Privacy-first positioning: clear statement that audio is not used for training

**Defer to v1.1:**
- AI post-processing commands (Claude API — rephrase, translate, shorten)
- Silence auto-stop / VAD-based auto-end
- Offline whisper.cpp fallback (complex model bundling, 100x slower on Windows)
- App-aware insertion cascade (Accessibility API for supported apps)

**Do not build:**
- Transcript history / search (let target apps own history)
- Real-time streaming transcription (batch after recording stop is fine)
- Built-in note-taking or canvas (text goes into whatever app is focused)
- Wake word / always-on mic (privacy risk, wrong activation model for desktop)

---

## Key Architecture Decisions

1. **All side effects in Rust backend.** Audio, clipboard, key simulation, API calls — none of this touches the frontend. Frontend is a passive display layer that invokes commands and listens to events.

2. **IPC pattern:** Commands (Frontend → Backend) for user-initiated actions. Events (Backend → Frontend) for state changes. Do not use a single monolithic command for the full pipeline.

3. **In-memory audio only.** Keep PCM as `Vec<f32>` in memory; encode to WAV in memory before API upload. No disk writes except for whisper.cpp which requires a file path.

4. **Recording state in Rust AppState.** `Arc<Mutex<RecordingState>>` via `.manage()`. Frontend mirrors state via events — never owns it.

5. **Clipboard save/restore wrapper.** Every paste operation must: save existing clipboard → write transcribed text → simulate paste → restore original after 50-100ms delay.

6. **Audio format must be explicitly configured:** 16kHz, mono, 16-bit PCM WAV. Not the browser or OS default. This is both a correctness requirement (Whisper quality) and a file-size requirement (25MB API limit).

7. **Text injection method:** Clipboard write + enigo paste simulation. Not accessibility APIs (fragile, app-specific). Known limitations (terminal, password managers) are documented, not hacked around.

8. **Phase dependencies:** Foundation (tray + settings) → Recording Core (hotkey + audio) → Cloud STT (Whisper + paste). Each phase delivers a runnable increment. Phases 1-3 = MVP.

---

## Critical Pitfalls

### 1. Clipboard Clobber — Silent Data Loss
Overwriting user's clipboard before paste destroys their copied content. Users lose URLs, passwords, code.
**Mitigation:** Always save clipboard before writing. Restore after 50-100ms post-paste. Required from day one.

### 2. macOS Microphone Permission Not Re-Prompted After Denial
Missing `NSMicrophoneUsageDescription` in Info.plist = silent empty audio, no system dialog. Cached denial = app silently broken.
**Mitigation:** Add Info.plist entry; check permission status before every recording attempt; deep-link to System Preferences if denied. Test on clean macOS account.

### 3. Whisper Hallucinations on Silence / Short Audio
Empty or sub-0.5s recordings produce fabricated text ("Thank you.", "Subtitles by...") that gets pasted into user documents.
**Mitigation:** VAD check before sending audio to Whisper. Minimum recording duration gate (0.5s above noise floor). Discard output matching known hallucination patterns. This is a correctness requirement, not polish.

### 4. Global Hotkey Registration Silently Fails
If default hotkey conflicts with Spotlight, Slack, VS Code — registration succeeds but hotkey does not fire. User sees no error.
**Mitigation:** Always handle registration errors explicitly. Show visible notification on failure. Provide hotkey remapping UI from day one. Default to an unusual combo (`Ctrl+Shift+Alt+R`).

### 5. macOS Accessibility Permission Required for Paste Simulation
`enigo` requires Accessibility permission to simulate Cmd+V. Silently does nothing without it in production builds.
**Mitigation:** Check Accessibility permission on first launch. Onboarding screen with deep-link to System Preferences. Do not enable hotkey until both Microphone and Accessibility are granted.

**Additional moderate pitfalls to track:**
- Windows: closing the main window kills the process — intercept `CloseRequested`, hide to tray instead
- whisper.cpp is 100x slower on Windows without AVX2+OpenBLAS — offline mode is Mac-first for v1
- Tauri cannot cross-compile — CI with per-platform runners required before first distribution milestone
- macOS notarization ($99/yr Apple Developer account) must be set up before any team rollout

---

## Build Order Recommendation

**Phase 1 — Foundation**
Tauri scaffold + system tray + settings UI + ConfigStore. No audio. Validates project runs on both platforms, establishes IPC pattern, permission flows, tray-only lifecycle (Windows close-to-tray). Must include: hotkey remapping UI, onboarding with permission checks.

**Phase 2 — Recording Core**
HotkeyManager (tauri-plugin-global-shortcut) + AudioCapture (cpal, 16kHz mono WAV). IPC: start_recording / stop_recording. Frontend waveform/status indicator. VAD gate and minimum duration check. Validates mic pipeline on both platforms.

**Phase 3 — Cloud STT + Paste (MVP complete)**
STTRouter cloud path (Whisper API via reqwest) + TextInjector (clipboard save/restore + enigo paste simulation). End-to-end: hotkey → audio → Whisper API → text injected. Clipboard clobber protection required here. This is the shippable MVP.

**Phase 4 — AI Commands**
Claude API integration (AICommandHandler). Command selection UI in tray popup. Depends on Phase 3 transcript output being available.

**Phase 5 — Offline Fallback**
whisper.cpp local path via whisper-rs. Model download/management UI. Mac-first (Metal GPU). Windows: base/tiny model only with speed warning. Needs phase-specific research for Tauri binary bundling.

**Phase 6 — Distribution + Polish**
macOS notarization pipeline (GitHub Actions, tauri-action). Windows signing (Azure Key Vault or EV cert). Error handling polish. Permission prompts. Universal macOS binary (Intel + Apple Silicon). App-aware insertion cascade (optional).

---

## Open Questions for Phase Planning

1. **Windows text insertion reliability:** Does enigo paste simulation work reliably in the target apps (Slack, VS Code, browser)? Needs validation in Phase 3 before committing to it as the primary method.

2. **Offline model distribution strategy:** Should whisper.cpp model files be bundled in the installer (~40-140MB) or downloaded on first use? Affects installer size, cold-start UX, and offline capability definition.

3. **whisper.cpp build pipeline:** Tauri sidecar binary approach vs whisper-rs crate linkage? Phase 5 needs dedicated research — the Tauri sidecar pattern and per-platform CI build requirements are non-trivial.

4. **Tray icon macOS bug (#13770):** Active Tauri bug where tray icon occasionally does not appear. Use `iconAsTemplate: false` + explicit PNG as workaround. Monitor Tauri releases.

5. **Hotkey hold-to-record implementation:** Toggle mode is straightforward. Hold-to-record requires tracking keydown vs keyup events — verify `tauri-plugin-global-shortcut` supports this or if rdev is needed (with the focus bug caveat).

6. **API key security:** `tauri-plugin-store` writes to a plain JSON file. For API key storage, should this be encrypted or delegated to the OS keychain (macOS Keychain, Windows Credential Store)? Assess in Phase 1.

---

## Confidence Assessment

| Area | Confidence | Notes |
|------|-----------|-------|
| Stack choices | HIGH | All primary crates verified against official docs and reference implementations |
| MVP feature scope | HIGH | Consistent across 5+ competitor apps analyzed |
| Architecture patterns | HIGH | Verified against Tauri v2 official docs + Pluely, Pothook reference apps |
| Critical pitfalls | HIGH | Most verified via GitHub issues (whisper.cpp #936, #1724, Tauri #11951) |
| whisper.cpp offline complexity | MEDIUM | Windows performance and Tauri bundling specifics need phase-specific validation |
| AI command UX patterns | MEDIUM | Feature exists in competitors but best-practice UX less documented |
| Distribution pipeline | MEDIUM | Known process, but signing/notarization setup details need phase-specific research |

**Overall: HIGH confidence for Phases 1-3. MEDIUM for Phases 4-6 (each needs phase-specific research before execution).**
