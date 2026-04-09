# Phase 6: Offline Fallback - Context

**Gathered:** 2026-04-09
**Status:** Ready for planning

<domain>
## Phase Boundary

Add local whisper.cpp STT via whisper-rs as offline fallback. User can choose STT engine (Cloud/Local/Auto). First-time local use triggers model download with progress. Windows gets a speed warning for non-AVX2 hardware.

</domain>

<decisions>
## Implementation Decisions

### Engine Selection
- **D-01:** Settings dropdown with 3 options: "Cloud (Whisper API)", "Lokal (whisper.cpp)", "Auto (lokal wenn offline)".
- **D-02:** Default: "Cloud". Stored in ConfigStore as `stt_mode`: `"cloud"` | `"local"` | `"auto"`.
- **D-03:** Auto mode: try cloud first, if network error within 3s → fallback to local. Requires local model to be downloaded.
- **D-04:** Engine selection stored in existing ConfigStore key `stt_mode` (already initialized in Phase 1).

### whisper-rs Integration
- **D-05:** Use whisper-rs crate (Rust bindings for whisper.cpp). Already in Cargo.toml from project research.
- **D-06:** Use "base" model (~140MB) as default. Good balance of quality and size.
- **D-07:** Model file stored in app data directory (Tauri's `app_data_dir()`). NOT bundled with installer.
- **D-08:** Transcription runs synchronously in a dedicated thread (same pattern as cpal capture in Phase 3).
- **D-09:** Feed the 16kHz mono PCM samples directly to whisper-rs (no WAV encoding needed for local — whisper-rs accepts raw f32 samples).

### Model Download
- **D-10:** First time user selects "Lokal" or "Auto" and model is missing → show download dialog.
- **D-11:** Download from HuggingFace (ggerganov/whisper.cpp model repository). Use reqwest with streaming.
- **D-12:** Show download progress in popup: percentage + MB downloaded / total MB.
- **D-13:** Download is cancellable. If cancelled, revert to Cloud mode.
- **D-14:** After download, verify file size matches expected. If corrupt, delete and re-prompt.

### Windows Speed Warning
- **D-15:** On Windows, when user selects "Lokal", show warning: "Lokale Transkription kann auf diesem Geraet langsam sein (10-30x langsamer als Cloud)."
- **D-16:** Warning is dismissable and only shown once per session (not every time).

### Claude's Discretion
- Exact whisper-rs API calls and configuration
- Download progress UI design
- Whether to use "tiny" model option for faster but lower quality

</decisions>

<canonical_refs>
## Canonical References

### Existing Code
- `src-tauri/src/stt/mod.rs` — Cloud STT (extend with local path)
- `src-tauri/src/recording/mod.rs` — WAV buffer + raw PCM samples
- `src-tauri/src/lib.rs` — Tauri commands, managed state
- `src/components/SettingsPage.tsx` — STT mode dropdown location
- `src/components/TrayPopup.tsx` — Progress/status display

### Research
- `.planning/research/STACK.md` — whisper-rs 0.16.0, model sizes
- `.planning/research/PITFALLS.md` — whisper.cpp 100x slowdown on Windows without optimization

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `stt/mod.rs` has `call_whisper_api()` — add `call_whisper_local()` alongside
- `recording/mod.rs` already produces 16kHz mono samples — feed directly to whisper-rs
- ConfigStore `stt_mode` key exists since Phase 1
- reqwest for streaming download (already in deps)

### Integration Points
- `transcribe_and_inject` in lib.rs needs router: check `stt_mode` → call cloud or local
- SettingsPage needs engine dropdown (extend existing)
- Model download → new Tauri command with event-based progress

</code_context>

<specifics>
## Specific Ideas

No specific requirements beyond decisions.

</specifics>

<deferred>
## Deferred Ideas

- Multiple model sizes (tiny/small/medium/large) — v2
- GPU acceleration toggle — v2
- Model management UI (delete/re-download) — v2

</deferred>

---

*Phase: 06-offline-fallback*
*Context gathered: 2026-04-09*
