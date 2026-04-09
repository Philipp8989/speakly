# Phase 4: Cloud STT + Text Injection (MVP) - Context

**Gathered:** 2026-04-09
**Status:** Ready for planning

<domain>
## Phase Boundary

Connect the recording pipeline (Phase 3) to OpenAI Whisper API for transcription, then inject the text into the active text field via clipboard + paste simulation. After this phase, Speakly is a functional MVP: hotkey → record → transcribe → paste.

</domain>

<decisions>
## Implementation Decisions

### Whisper API Integration
- **D-01:** Use OpenAI Whisper API endpoint `POST /v1/audio/transcriptions` with model `whisper-1`.
- **D-02:** Send the WAV buffer directly as multipart form data from Rust (reqwest). No temp file on disk.
- **D-03:** Response format: `json` (returns `{ "text": "..." }`). Parse text field only.
- **D-04:** If API key missing or invalid, show error in popup ("API-Key fehlt oder ungueltig") and do NOT paste anything.
- **D-05:** On network error or timeout (30s), show "Netzwerkfehler — Transkription fehlgeschlagen" in popup.
- **D-06:** Empty transcription result (Whisper returns empty string) → discard, don't paste. Show brief "Keine Sprache erkannt".

### Language Handling
- **D-07:** Read `language` from ConfigStore. If set to specific language (e.g., "de", "en"), pass as `language` parameter to Whisper API.
- **D-08:** If `language` is "auto" or unset, omit the `language` parameter — Whisper auto-detects.
- **D-09:** Add `language` dropdown to Settings page (populated from Whisper's supported languages). Default: "auto".

### Clipboard Save/Restore
- **D-10:** Before pasting transcript: save current clipboard content (text only — images/files not supported in v1).
- **D-11:** Write transcript text to clipboard via tauri-plugin-clipboard-manager.
- **D-12:** After writing, simulate paste via enigo: Cmd+V on macOS, Ctrl+V on Windows.
- **D-13:** After paste (500ms delay), restore original clipboard content. If restore fails, log warning but don't block.
- **D-14:** Entire clipboard operation (save → write → paste → restore) must complete within 2 seconds.

### Text Injection
- **D-15:** v1 uses clipboard + paste simulation only. No Accessibility API cascade — too complex for MVP.
- **D-16:** enigo crate for key simulation. Use `Key::Meta` + `Key::V` on macOS, `Key::Control` + `Key::V` on Windows.
- **D-17:** Small delay (50ms) between clipboard write and paste simulation to ensure clipboard is ready.
- **D-18:** If enigo paste fails (permission denied on macOS), show "Accessibility-Berechtigung benoetigt" with deep-link to System Preferences.

### Processing UX
- **D-19:** During Whisper API call, tray popup shows "Verarbeite..." with processing state indicator.
- **D-20:** Tray icon shows processing state (different from idle and recording).
- **D-21:** On successful paste, briefly show the transcribed text in the popup preview area (2 seconds), then return to idle.
- **D-22:** All error messages auto-dismiss after 5 seconds or on next hotkey press.

### End-to-End Flow
- **D-23:** Full flow: hotkey stop → tray shows "Verarbeite..." → Whisper API call → clipboard save → write transcript → paste → restore clipboard → tray shows transcript briefly → idle.
- **D-24:** If recording was discarded by VAD (Phase 3), skip the entire STT pipeline. No API call, no paste.

### Claude's Discretion
- Exact enigo API calls and error handling details
- Processing spinner/animation design
- Transcript preview truncation (if text too long for popup)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Existing Code
- `src-tauri/src/recording/mod.rs` — Recording state, WAV buffer output
- `src-tauri/src/recording/wav_encoder.rs` — WAV encoding (in-memory)
- `src-tauri/src/lib.rs` — Tauri commands, plugin setup, managed state
- `src-tauri/Cargo.toml` — reqwest + enigo already listed
- `src/App.tsx` — Event listeners for recording state changes
- `src/components/TrayPopup.tsx` — Popup UI to extend with processing/transcript states

### Research
- `.planning/research/STACK.md` — enigo 0.2+ paste simulation pattern, clipboard-manager API
- `.planning/research/PITFALLS.md` — Clipboard clobber (#1 trust-destroying bug), Accessibility permission

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `reqwest` already in Cargo.toml (used for API key validation in Phase 2)
- `tauri-plugin-clipboard-manager` already installed and permitted
- Recording module outputs WAV `Vec<u8>` buffer ready for Whisper API
- Event system for backend→frontend communication established
- TrayPopup has status indicator + transcript preview area

### Established Patterns
- Tauri commands for API calls from Rust backend
- ConfigStore for API keys and preferences
- Event-based state updates (recording_state_changed pattern from Phase 3)

### Integration Points
- After `stop_capture()` returns WAV buffer → feed into new `transcribe()` command
- After transcription → new `inject_text()` command handles clipboard + paste
- TrayPopup needs new states: processing, transcript_preview, error
- Language setting needs UI in SettingsPage (dropdown)

</code_context>

<specifics>
## Specific Ideas

- The paste must feel instant — user should see their text appear in the target app within 2-3 seconds of stopping recording
- Clipboard restore is critical — users WILL notice if their clipboard is clobbered
- Error messages should be helpful, not technical — "API-Key fehlt" not "HTTP 401 Unauthorized"

</specifics>

<deferred>
## Deferred Ideas

- Accessibility API cascade for text injection — v2 feature, too complex for MVP
- Image/file clipboard save/restore — text only in v1
- Streaming transcription — batch after stop is fine for v1

</deferred>

---

*Phase: 04-cloud-stt-text-injection-mvp*
*Context gathered: 2026-04-09*
