# Phase 4: Cloud STT + Text Injection (MVP) - Research

**Researched:** 2026-04-09
**Domain:** OpenAI Whisper API (reqwest multipart), enigo paste simulation, tauri-plugin-clipboard-manager save/restore
**Confidence:** HIGH (all core decisions already locked, codebase verified)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Whisper API Integration**
- D-01: Use OpenAI Whisper API `POST /v1/audio/transcriptions` with model `whisper-1`
- D-02: Send WAV buffer directly as multipart form data from Rust (reqwest). No temp file on disk
- D-03: Response format: `json` (returns `{ "text": "..." }`). Parse text field only
- D-04: If API key missing or invalid, show error in popup ("API-Key fehlt oder ungueltig"), do NOT paste
- D-05: On network error or timeout (30s), show "Netzwerkfehler — Transkription fehlgeschlagen" in popup
- D-06: Empty transcription result → discard, don't paste. Show "Keine Sprache erkannt"

**Language Handling**
- D-07: Read `language` from ConfigStore. If set (e.g. "de"), pass as `language` parameter to Whisper
- D-08: If `language` is "auto" or unset, omit the `language` parameter entirely
- D-09: Add `language` dropdown to Settings page. Default: "auto"

**Clipboard Save/Restore**
- D-10: Before pasting: save current clipboard content (text only — images/files not supported in v1)
- D-11: Write transcript text to clipboard via tauri-plugin-clipboard-manager
- D-12: Simulate paste via enigo: Cmd+V on macOS, Ctrl+V on Windows
- D-13: After paste (500ms delay), restore original clipboard. If restore fails, log warning but don't block
- D-14: Entire clipboard operation (save → write → paste → restore) must complete within 2 seconds

**Text Injection**
- D-15: v1 uses clipboard + paste simulation only. No Accessibility API cascade
- D-16: enigo crate for key simulation. `Key::Meta` + `Key::V` on macOS, `Key::Control` + `Key::V` on Windows
- D-17: 50ms delay between clipboard write and paste simulation
- D-18: If enigo paste fails (permission denied), show "Accessibility-Berechtigung benoetigt" with deep-link to System Preferences

**Processing UX**
- D-19: During Whisper API call, tray popup shows "Verarbeite..." with processing state indicator
- D-20: Tray icon shows processing state (different from idle and recording)
- D-21: On successful paste, briefly show transcribed text in popup preview area (2 seconds), then idle
- D-22: All error messages auto-dismiss after 5 seconds or on next hotkey press

**End-to-End Flow**
- D-23: hotkey stop → "Verarbeite..." → Whisper API → clipboard save → write transcript → paste → restore clipboard → show transcript briefly → idle
- D-24: If recording was discarded by VAD (Phase 3), skip entire STT pipeline. No API call, no paste.

### Claude's Discretion
- Exact enigo API calls and error handling details
- Processing spinner/animation design
- Transcript preview truncation (if text too long for popup)

### Deferred Ideas (OUT OF SCOPE)
- Accessibility API cascade for text injection (v2)
- Image/file clipboard save/restore (text only in v1)
- Streaming transcription (batch after stop is fine for v1)
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| STT-01 | User bekommt Transkription via OpenAI Whisper API (Cloud) | reqwest multipart POST to `/v1/audio/transcriptions`; WAV buffer already produced by Phase 3 wav_encoder |
| STT-02 | Transkription enthaelt automatische Punktuation und Gross-/Kleinschreibung | Whisper API default behavior with `response_format: json`; no extra params needed |
| STT-03 | Sprache wird automatisch erkannt (50+ Sprachen) | Omit `language` param → Whisper auto-detects; pass ISO 639-1 code when user sets preference |
| APP-03 | User kann Sprach-Praeferenz setzen oder Auto-Detect nutzen | `language` key already in ConfigStore (set to "auto" in setup); add dropdown to SettingsPage |
| INS-01 | Transkribierter Text wird automatisch ins aktive Textfeld eingefuegt | enigo Cmd+V / Ctrl+V after clipboard write |
| INS-02 | Bisheriger Clipboard-Inhalt wird vor dem Einfuegen gesichert und danach wiederhergestellt | tauri-plugin-clipboard-manager read_text → save → write transcript → paste → restore |
| INS-03 | Text-Einfuegen nutzt App-aware Cascade (Accessibility API -> Clipboard+Paste -> Keystroke) | v1 scope: clipboard+paste only (D-15); INS-03 is partially met — full cascade is v2 |
</phase_requirements>

---

## Summary

Phase 4 wires three existing pieces into a complete MVP pipeline: (1) the WAV buffer produced by Phase 3's `stop_capture()`, (2) the OpenAI Whisper API via the already-present `reqwest` crate, and (3) text injection via `enigo` + `tauri-plugin-clipboard-manager`. The recording module is complete and tested. The HTTP client is present. The only new Rust dependencies are `enigo` and `tauri-plugin-clipboard-manager` — neither is in Cargo.toml yet despite the CONTEXT.md claim.

The main implementation risks are: (a) clipboard clobber if the save/restore timing is wrong, (b) macOS Accessibility permission silently blocking enigo, and (c) the 500ms restore delay being too short for async-clipboard apps like Chrome. The UX layer requires TrayPopup to grow three new states: processing, transcript_preview, and error.

**Primary recommendation:** Implement the pipeline as a single Tauri command `transcribe_and_inject(wav_bytes)` that owns the full flow (Whisper call → clipboard save → write → paste → restore), emitting progress events to the frontend at each stage. Keep clipboard save/restore entirely in Rust (arboard or plugin) to avoid JS↔Rust round-trips during the timing-sensitive paste window.

---

## Standard Stack

### Core — New Dependencies for Phase 4

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| enigo | 0.6.1 | Keyboard simulation (Cmd+V / Ctrl+V) | Cross-platform, maintained, correct macOS/Windows support. rdev has Sonoma bugs. [VERIFIED: crates.io] |
| tauri-plugin-clipboard-manager | 2.3.2 | Read/write clipboard text | Official Tauri plugin, already chosen in STACK.md. [VERIFIED: crates.io + npm registry] |
| @tauri-apps/plugin-clipboard-manager | 2.3.2 | Frontend JS bindings (needed if read/write from TS) | Matches Rust plugin version. [VERIFIED: npm registry] |

### Already Present (no changes needed)

| Library | Version | Status |
|---------|---------|--------|
| reqwest | 0.12 | In Cargo.toml — has `json` + `multipart` features already |
| tokio | 1 | In Cargo.toml — `rt-multi-thread` feature present |
| tauri-plugin-store | 2 | In Cargo.toml — ConfigStore for API key + language |
| serde_json | 1 | In Cargo.toml |

### Critical Gap Found

**enigo and tauri-plugin-clipboard-manager are NOT in Cargo.toml.** CONTEXT.md states "reqwest + enigo already listed" — this is incorrect as of the current codebase. Both must be added in Wave 0 of the plan.

```bash
# Rust
cargo add enigo@0.6
cargo add tauri-plugin-clipboard-manager@2

# Frontend (only if reading clipboard from JS)
pnpm add @tauri-apps/plugin-clipboard-manager
```

**Version verification:** [VERIFIED: crates.io 2026-04-09]
- enigo: 0.6.1 (latest stable)
- tauri-plugin-clipboard-manager: 2.3.2 (Rust) / 2.3.2 (npm)

---

## Architecture Patterns

### Recommended Project Structure (new files this phase)

```
src-tauri/src/
├── lib.rs                      # Add: transcribe_and_inject command + register
├── recording/                  # Unchanged (Phase 3 complete)
├── stt/
│   └── mod.rs                  # New: call_whisper_api() function
└── inject/
    └── mod.rs                  # New: save_clipboard(), inject_text(), restore_clipboard()

src/components/
├── TrayPopup.tsx               # Extend: processing + transcript_preview + error states
└── SettingsPage.tsx            # Extend: language dropdown (APP-03)
```

### Pattern 1: Single Tauri Command Owns Full Pipeline

**What:** One `#[tauri::command] async fn transcribe_and_inject(...)` handles the entire flow from WAV bytes to paste completion.

**When to use:** When a sequence of steps must be atomic from the user's perspective (no partial states exposed).

```rust
// src-tauri/src/lib.rs — new command
#[tauri::command]
async fn transcribe_and_inject(
    app: tauri::AppHandle,
    state: tauri::State<'_, RecordingState>,
) -> Result<String, String> {
    // 1. WAV buffer aus State holen
    let wav = {
        let lock = state.wav_buffer.lock().unwrap();
        lock.clone().ok_or("Kein WAV-Puffer vorhanden")?
    };

    // 2. Processing-State an Frontend senden
    let _ = app.emit("transcription_state_changed", serde_json::json!({
        "state": "processing"
    }));

    // 3. Whisper API aufrufen
    let text = crate::stt::call_whisper_api(&app, wav).await
        .map_err(|e| { let _ = app.emit("transcription_state_changed", serde_json::json!({"state": "error", "message": e})); e })?;

    if text.trim().is_empty() {
        let _ = app.emit("transcription_state_changed", serde_json::json!({
            "state": "error", "message": "Keine Sprache erkannt"
        }));
        return Ok(String::new());
    }

    // 4. Text in aktives Feld einfuegen (save → write → paste → restore)
    crate::inject::inject_text(&app, &text).await?;

    // 5. Erfolg an Frontend melden
    let _ = app.emit("transcription_state_changed", serde_json::json!({
        "state": "done", "text": text.trim()
    }));

    Ok(text)
}
```

[ASSUMED — pattern based on established Phase 3 event pattern]

### Pattern 2: Whisper API Call via reqwest Multipart

```rust
// src-tauri/src/stt/mod.rs
use reqwest::multipart;

pub async fn call_whisper_api(app: &tauri::AppHandle, wav_bytes: Vec<u8>) -> Result<String, String> {
    use tauri_plugin_store::StoreExt;

    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    let api_key = store.get("openai_api_key")
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_default();

    if api_key.trim().is_empty() {
        return Err("API-Key fehlt oder ungueltig".into());
    }

    let language = store.get("language")
        .and_then(|v| v.as_str().map(|s| s.to_string()));

    // Multipart-Formular aufbauen
    let file_part = multipart::Part::bytes(wav_bytes)
        .file_name("audio.wav")
        .mime_str("audio/wav")
        .map_err(|e| e.to_string())?;

    let mut form = multipart::Form::new()
        .part("file", file_part)
        .text("model", "whisper-1")
        .text("response_format", "json");

    // language-Parameter nur hinzufuegen wenn explizit gesetzt (nicht "auto")
    if let Some(lang) = language {
        if lang != "auto" && !lang.is_empty() {
            form = form.text("language", lang);
        }
    }

    let client = reqwest::Client::new();
    let resp = client
        .post("https://api.openai.com/v1/audio/transcriptions")
        .header("Authorization", format!("Bearer {}", api_key.trim()))
        .multipart(form)
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await
        .map_err(|e| {
            if e.is_timeout() {
                "Netzwerkfehler — Transkription fehlgeschlagen".to_string()
            } else {
                "Netzwerkfehler — Transkription fehlgeschlagen".to_string()
            }
        })?;

    if resp.status() == 401 || resp.status() == 403 {
        return Err("API-Key fehlt oder ungueltig".into());
    }

    if !resp.status().is_success() {
        return Err("Netzwerkfehler — Transkription fehlgeschlagen".into());
    }

    let body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    let text = body["text"].as_str().unwrap_or("").to_string();
    Ok(text)
}
```

[ASSUMED — based on reqwest multipart docs and OpenAI API spec]

### Pattern 3: Clipboard Save/Restore with Timing

```rust
// src-tauri/src/inject/mod.rs
use enigo::{Enigo, Key, Direction, Settings, Keyboard};
use tauri_plugin_clipboard_manager::ClipboardExt;

pub async fn inject_text(app: &tauri::AppHandle, text: &str) -> Result<(), String> {
    // 1. Aktuellen Clipboard-Inhalt sichern
    let original = app.clipboard().read_text().ok();

    // 2. Transkription in Clipboard schreiben
    app.clipboard().write_text(text.to_string())
        .map_err(|e| e.to_string())?;

    // 3. Kurz warten (D-17: 50ms) damit Clipboard bereit ist
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    // 4. Paste simulieren (Cmd+V macOS / Ctrl+V Windows)
    let mut enigo = Enigo::new(&Settings::default())
        .map_err(|e| format!("Accessibility-Berechtigung benoetigt: {}", e))?;

    #[cfg(target_os = "macos")]
    {
        let _ = enigo.key(Key::Meta, Direction::Press);
        let _ = enigo.key(Key::Unicode('v'), Direction::Click);
        let _ = enigo.key(Key::Meta, Direction::Release);
    }
    #[cfg(target_os = "windows")]
    {
        let _ = enigo.key(Key::Control, Direction::Press);
        let _ = enigo.key(Key::Unicode('v'), Direction::Click);
        let _ = enigo.key(Key::Control, Direction::Release);
    }

    // 5. Warten (D-13: 500ms) damit Zielapp Paste verarbeiten kann
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // 6. Originalen Clipboard wiederherstellen
    if let Some(orig) = original {
        if let Err(e) = app.clipboard().write_text(orig) {
            eprintln!("Warnung: Clipboard-Wiederherstellung fehlgeschlagen: {}", e);
            // Kein Fehler zurueckgeben — User-Flow hat bereits geklappt
        }
    }

    Ok(())
}
```

[ASSUMED — enigo 0.6 API shape based on GitHub docs; exact method signatures need verification]

### Pattern 4: TrayPopup State Machine

The TrayPopup needs to drive from a single `AppState` enum. Current state is static (Phase 1 placeholder).

**States:** `idle | recording | processing | transcript_preview(text) | error(message)`

Frontend listens to `transcription_state_changed` and `recording_state_changed` events (established in Phase 3).

```typescript
// src/components/TrayPopup.tsx — new state type
type AppState =
  | { kind: 'idle' }
  | { kind: 'recording' }
  | { kind: 'processing' }
  | { kind: 'transcript'; text: string }
  | { kind: 'error'; message: string };
```

[ASSUMED — TypeScript pattern based on Phase 3 event structure]

### Pattern 5: Language Dropdown for Settings (APP-03)

`language` key is already in ConfigStore, defaulting to `"auto"`. The Settings page needs a `<select>` populated from Whisper's supported language list. The value is ISO 639-1 code or `"auto"`.

**Whisper supported languages (partial):** de, en, fr, es, it, pt, nl, pl, ru, zh, ja, ko, ar, tr — plus ~40 more. [CITED: platform.openai.com/docs/guides/speech-to-text]

### Anti-Patterns to Avoid

- **Fire-and-forget clipboard write:** Never write to clipboard without saving first — even in "fast" paths where restore "probably isn't needed"
- **JS-side clipboard round-trip:** Don't read/write clipboard via the JS `@tauri-apps/plugin-clipboard-manager` if the entire inject flow is in Rust. JS round-trips add latency during the timing-sensitive paste window
- **Swallowing enigo init errors:** If `Enigo::new()` fails (Accessibility denied on macOS), surface it explicitly — it will silently do nothing otherwise
- **Calling `transcribe_and_inject` from the hotkey handler directly:** The hotkey handler fires on every key event including the toggle-stop. The stop_capture result should emit `recording_stopped` which the frontend catches to invoke `transcribe_and_inject` — keeps the hot path clean

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Clipboard read/write | Custom arboard bindings | tauri-plugin-clipboard-manager | Official plugin, cross-platform, already chosen |
| Key simulation | CGEventPost (macOS) / SendInput (Windows) unsafe bindings | enigo 0.6 | Wraps platform APIs, maintained, avoids unsafe boilerplate |
| HTTP multipart | Manual boundary construction | reqwest multipart feature (already in Cargo.toml) | Already present with `multipart` feature |
| API key retrieval | Custom file I/O | tauri-plugin-store StoreExt trait | Already in Cargo.toml, established pattern from Phase 2 |

---

## Common Pitfalls

### Pitfall 1: Clipboard Clobber (Critical — #1 Trust Destroyer)
**What goes wrong:** Transcript overwrites clipboard; original content (copied URL, code, password) is lost permanently.
**Why it happens:** Paste flow omits the save step, or the restore runs before the target app finishes reading the pasted content.
**How to avoid:** Save clipboard BEFORE writing transcript. Wait 500ms AFTER paste before restoring (D-13). This is already locked as D-10–D-13.
**Warning signs:** Clipboard content disappears after a dictation. Any code path that calls `write_text` without a preceding `read_text` is wrong.

### Pitfall 2: macOS Accessibility Permission Blocks enigo Silently
**What goes wrong:** `Enigo::new()` succeeds but key events are silently swallowed. Paste does nothing. No error returned.
**Why it happens:** macOS blocks untrusted apps from sending keystrokes to other apps even when enigo initializes without error. Error only surfaces when sending the first key event.
**How to avoid:** Check for Accessibility permission before calling inject_text. `tauri-plugin-macos-permissions` is already in Cargo.toml (macOS target). On failure, emit error "Accessibility-Berechtigung benoetigt" with deep-link (D-18).
**Warning signs:** Paste works in terminal/dev but not in production builds or on fresh user machines.

### Pitfall 3: Recording Discarded but STT Pipeline Triggered
**What goes wrong:** `recording_discarded` event fires (VAD rejected audio) but frontend still calls `transcribe_and_inject`.
**Why it happens:** Frontend listens to `recording_stopped` and `recording_discarded` as separate events; if only one handler triggers the command, a missed check leads to API call on empty audio.
**How to avoid:** Only trigger `transcribe_and_inject` from `recording_stopped` (wav_ready: true), never from `recording_discarded`. D-24 locks this.

### Pitfall 4: 500ms Restore Delay Too Short for Chrome/Electron
**What goes wrong:** Chromium-based apps read clipboard asynchronously. The 500ms restore window may fire before the paste has been consumed, replacing transcript text with old clipboard content in the target field.
**Why it happens:** Chrome's async clipboard API vs. the synchronous paste simulation. Timing is app-dependent.
**How to avoid:** The 500ms delay (D-13) is a reasonable v1 compromise. Log if restore fails silently. Document known limitation. Do not increase to >1s as that degrades UX.
**Warning signs:** Pasted text briefly appears then gets overwritten, or paste area shows old clipboard content instead of transcript.

### Pitfall 5: enigo API Changed Between 0.2 and 0.6
**What goes wrong:** STACK.md code examples show the enigo 0.2 API (`Key::Meta`, `Key::Unicode`). enigo 0.6 restructured the API.
**Why it happens:** The research docs were written with an older API shape. [ASSUMED — needs verification against enigo 0.6 changelog]
**How to avoid:** Verify exact method signatures from `cargo doc --open` or enigo GitHub after adding the dependency. The `Keyboard` trait and `Direction` enum are the correct 0.6+ pattern.

### Pitfall 6: reqwest multipart Part Requires Mime Type
**What goes wrong:** `Part::bytes(wav_bytes).file_name("audio.wav")` without `.mime_str("audio/wav")` — Whisper API returns 400 or processes incorrectly.
**Why it happens:** OpenAI validates the `Content-Type` of the file part in the multipart body.
**How to avoid:** Always set `.mime_str("audio/wav")` on the file part (shown in Pattern 2 above).

---

## Code Examples

### Trigger Point: recording_stopped → transcribe_and_inject

The integration point between Phase 3 and Phase 4 is the `recording_stopped` event. Frontend catches it and invokes the new command.

```typescript
// src/App.tsx — in useEffect, alongside existing listeners
listen<{ wav_ready: boolean; duration_ms: number }>('recording_stopped', (event) => {
  if (event.payload.wav_ready) {
    invoke('transcribe_and_inject').catch(console.error);
  }
}).then(ul => unlisteners.push(ul));
```

[ASSUMED — matches established event pattern from Phase 3]

### Event: transcription_state_changed Payload Shape

```typescript
type TranscriptionStatePayload =
  | { state: 'processing' }
  | { state: 'done'; text: string }
  | { state: 'error'; message: string };
```

### Register Command in lib.rs invoke_handler

```rust
.invoke_handler(tauri::generate_handler![
    validate_openai_key,
    validate_anthropic_key,
    list_audio_input_devices,
    check_macos_permissions,
    toggle_recording,
    stop_recording_hold,
    transcribe_and_inject,   // NEW Phase 4
])
```

### Register clipboard plugin in lib.rs builder

```rust
.plugin(tauri_plugin_clipboard_manager::init())  // NEW Phase 4
```

---

## State of the Art

| Old Approach | Current Approach | Impact |
|--------------|-----------------|--------|
| enigo 0.2 API (`Enigo::new()`, `key_click`) | enigo 0.6 API (`Enigo::new(&Settings::default())`, `Keyboard` trait) | Code examples in STACK.md use older API shape — verify exact calls |
| Whisper `whisper-1` only | `gpt-4o-transcribe` also available (higher quality) | D-01 locks `whisper-1` for v1; gpt-4o-transcribe is a future upgrade path |

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `Enigo::new(&Settings::default())` is correct constructor in 0.6.x | Pattern 3, Pitfall 5 | Compile error — easy to fix during implementation |
| A2 | `app.clipboard().read_text()` / `write_text()` is the correct ClipboardExt API shape for tauri-plugin-clipboard-manager 2.3.2 | Pattern 3 | Compile error — check docs.rs after adding dependency |
| A3 | `Key::Meta` + `Key::Unicode('v')` is correct for Cmd+V in enigo 0.6 | Pattern 3 | Wrong keys sent — paste fails silently on macOS |
| A4 | Whisper API auto-detects language when `language` param is omitted (not when set to empty string) | Pattern 2 | Incorrect behavior — could degrade transcription quality |
| A5 | `recording_stopped` fires with `wav_ready: true` only when VAD passes (verified in mod.rs stop_capture) | Code Examples | STT called on empty audio — verified FALSE, code is correct |

**Note on A5:** Verified in `src-tauri/src/recording/mod.rs` — `recording_stopped` is only emitted after successful VAD check and WAV encoding. `recording_discarded` fires when VAD rejects. A5 is VERIFIED, not assumed.

---

## Open Questions

1. **enigo 0.6 exact API for macOS Cmd+V**
   - What we know: enigo 0.6.1 is on crates.io, API changed from 0.2
   - What's unclear: Whether `Key::Meta` + `Key::Unicode('v')` vs. `Key::Command` + `Key::Unicode('v')` is correct
   - Recommendation: Add the dependency and check `cargo doc` output before writing inject code. Treat all enigo call patterns in this doc as A1/A3.

2. **ClipboardExt trait method names in tauri-plugin-clipboard-manager 2.3.2**
   - What we know: Plugin is official and documented
   - What's unclear: Whether `app.clipboard().read_text()` or `ClipboardManager::read_text(&app)` is the correct calling convention in Rust
   - Recommendation: Check docs.rs/tauri-plugin-clipboard-manager after adding dependency.

3. **INS-03 compliance with v1 scope**
   - What we know: INS-03 says "App-aware Cascade (Accessibility API -> Clipboard+Paste -> Keystroke)". D-15 locks v1 to clipboard+paste only.
   - What's unclear: Whether the requirements doc considers INS-03 partially met by v1 or fully deferred
   - Recommendation: Note in plan that clipboard+paste satisfies INS-03 for all common apps. Full cascade (AXUIElement) is Phase 6.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| enigo | Paste simulation | Not yet installed | 0.6.1 on crates.io | None — required |
| tauri-plugin-clipboard-manager (Rust) | Clipboard read/write | Not yet installed | 2.3.2 on crates.io | arboard crate (not recommended) |
| @tauri-apps/plugin-clipboard-manager (npm) | JS bindings | Not yet installed | 2.3.2 on npm | Not needed if clipboard handled entirely in Rust |
| reqwest (multipart) | Whisper API | Installed | 0.12 with multipart feature | Already present |
| OpenAI Whisper API | STT | External service | Current | None (cloud-only in v1) |

**Missing dependencies with no fallback:**
- enigo 0.6 — must be added to Cargo.toml before any inject code compiles
- tauri-plugin-clipboard-manager — must be added to Cargo.toml and registered as plugin

**Missing dependencies with fallback:**
- None

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` + `#[cfg(test)]` modules |
| Config file | None (standard Cargo test runner) |
| Quick run command | `cargo test -p speakly 2>&1` |
| Full suite command | `cargo test -p speakly 2>&1` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| STT-01 | Whisper API called with correct multipart form, returns text | unit (mock HTTP) | `cargo test stt::tests` | Wave 0 |
| STT-01 | API key missing → error string "API-Key fehlt oder ungueltig" | unit | `cargo test stt::tests::test_missing_api_key` | Wave 0 |
| STT-01 | HTTP 401 → error "API-Key fehlt oder ungueltig" | unit | `cargo test stt::tests::test_invalid_api_key` | Wave 0 |
| STT-01 | Timeout (30s) → error "Netzwerkfehler" | unit | `cargo test stt::tests::test_timeout` | Wave 0 |
| STT-02 | Punctuation/capitalization returned from API (Whisper default) | manual | N/A | N/A — API behavior |
| STT-03 | language="auto" → no `language` param in request | unit | `cargo test stt::tests::test_language_omitted` | Wave 0 |
| STT-03 | language="de" → `language=de` in request | unit | `cargo test stt::tests::test_language_param` | Wave 0 |
| APP-03 | language dropdown renders in SettingsPage | manual/visual | N/A | N/A |
| INS-01 | inject_text calls enigo after clipboard write | unit (mock enigo) | `cargo test inject::tests` | Wave 0 |
| INS-02 | Original clipboard restored after paste | unit | `cargo test inject::tests::test_clipboard_restore` | Wave 0 |
| INS-02 | Empty original clipboard → no restore attempted | unit | `cargo test inject::tests::test_no_original` | Wave 0 |
| D-06 | Empty transcription → no inject called | unit | `cargo test stt::tests::test_empty_result` | Wave 0 |
| D-24 | recording_discarded → transcribe_and_inject NOT called | integration/manual | N/A | N/A — event routing |

**Note on mock strategy:** HTTP calls to Whisper API cannot be tested against live API in unit tests. Use `wiremock` or a simple mock server pattern to return fixture responses. Clipboard and enigo calls should be wrapped in traits for testability.

### Sampling Rate
- **Per task commit:** `cargo test -p speakly 2>&1`
- **Per wave merge:** `cargo test -p speakly 2>&1`
- **Phase gate:** All tests green before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] `src-tauri/src/stt/mod.rs` — module + `call_whisper_api` + unit tests (STT-01, STT-03)
- [ ] `src-tauri/src/inject/mod.rs` — module + `inject_text` + unit tests (INS-01, INS-02)
- [ ] `src-tauri/src/stt/tests.rs` — HTTP mock tests for API key error paths
- [ ] Add `enigo` to Cargo.toml
- [ ] Add `tauri-plugin-clipboard-manager` to Cargo.toml

---

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | N/A |
| V3 Session Management | no | N/A |
| V4 Access Control | no | N/A |
| V5 Input Validation | yes | Trim whitespace from API key; reject empty strings before HTTP call (already in validate_openai_key pattern) |
| V6 Cryptography | no | API keys stored in plain JSON (tauri-plugin-store) — accepted in Phase 2 |

### Known Threat Patterns

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| API key logged accidentally | Information Disclosure | Never log api_key value — only log key length or masked value. Pattern established in validate_openai_key |
| Transcript injected into wrong window | Tampering | No mitigation possible via clipboard+paste approach — by design. Document as known limitation. |
| Clipboard sniffing by other apps | Information Disclosure | Cannot prevent — OS clipboard is shared. Restore minimizes exposure window (500ms). |

---

## Sources

### Primary (HIGH confidence)
- Cargo.toml (project root) — verified enigo and clipboard-manager are ABSENT
- `src-tauri/src/recording/mod.rs` — verified `stop_capture` returns `Option<Vec<u8>>` and emits `recording_stopped` / `recording_discarded`
- `src-tauri/src/lib.rs` — verified reqwest, plugin-store, event emission patterns
- `.planning/research/STACK.md` — paste flow pattern and enigo usage
- `.planning/research/PITFALLS.md` — clipboard clobber, Accessibility permission pitfalls
- `.planning/phases/04-cloud-stt-text-injection-mvp/04-CONTEXT.md` — 24 locked decisions

### Secondary (MEDIUM confidence)
- [VERIFIED: crates.io] enigo 0.6.1 is current stable
- [VERIFIED: crates.io] tauri-plugin-clipboard-manager 2.3.2 (Rust)
- [VERIFIED: npm registry] @tauri-apps/plugin-clipboard-manager 2.3.2
- [CITED: platform.openai.com/docs/guides/speech-to-text] Whisper language auto-detection via omitting `language` param

### Tertiary (LOW confidence — verify at implementation)
- enigo 0.6 exact API method signatures — verify from `cargo doc` after adding dependency [ASSUMED]
- ClipboardExt trait method names for tauri-plugin-clipboard-manager 2.3.x [ASSUMED]

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all libraries verified on registries; gap in Cargo.toml confirmed
- Architecture: MEDIUM — patterns follow established project conventions; enigo exact API [ASSUMED]
- Pitfalls: HIGH — clipboard clobber and Accessibility permission are documented, verified in PITFALLS.md

**Research date:** 2026-04-09
**Valid until:** 2026-05-09 (stable libraries, 30-day horizon)
