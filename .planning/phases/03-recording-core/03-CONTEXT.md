# Phase 3: Recording Core - Context

**Gathered:** 2026-04-09
**Status:** Ready for planning

<domain>
## Phase Boundary

Implement the full recording pipeline: global hotkey triggers audio capture via cpal, live waveform feedback in the popup, VAD gate discards bad audio. After this phase, pressing the hotkey produces a WAV buffer ready for STT (Phase 4).

</domain>

<decisions>
## Implementation Decisions

### Recording Modes
- **D-01:** Toggle mode: first press starts recording, second press stops. This is the primary mode.
- **D-02:** Hold-to-record (push-to-talk): hold hotkey starts, release stops. Secondary mode.
- **D-03:** Detect toggle vs hold via keydown/keyup timing from tauri-plugin-global-shortcut. If key held >300ms, treat as hold-to-record. If quick press+release, treat as toggle.
- **D-04:** Recording state managed in Rust (not frontend). Frontend receives state change events via Tauri event system.

### Audio Capture
- **D-05:** Use cpal crate for audio capture. Get default input device or user-selected device from ConfigStore `microphone_id`.
- **D-06:** Record at 16kHz mono PCM (i16). This is Whisper API's optimal input format.
- **D-07:** Encode captured PCM frames to WAV using hound crate. WAV buffer kept in memory (not written to disk).
- **D-08:** Audio capture runs in a dedicated thread (not async). Communicate with main thread via channels (std::sync::mpsc).

### VAD (Voice Activity Detection)
- **D-09:** Simple RMS energy threshold — no external VAD library needed for v1.
- **D-10:** Discard recordings where: duration < 0.5 seconds OR max RMS energy < configurable threshold.
- **D-11:** When discarded, emit a "recording_discarded" event to frontend (show brief "Zu kurz" message). Do NOT paste anything.

### Waveform / Visual Feedback
- **D-12:** During recording, send RMS audio level values from Rust to frontend via Tauri events at ~10-15 updates/sec.
- **D-13:** TrayPopup shows animated audio level bars (simple vertical bars, not full waveform). Height proportional to current RMS level.
- **D-14:** Tray icon state changes: idle → recording (different icon/color) → processing (after stop, before STT returns in Phase 4).

### Silence Auto-Stop
- **D-15:** Recording stops automatically after configurable silence duration (default: 3 seconds of continuous silence).
- **D-16:** Silence defined as RMS energy below threshold for N consecutive frames.
- **D-17:** Auto-stop duration configurable in settings (add to ConfigStore: `silence_timeout_ms`, default 3000).

### Claude's Discretion
- Exact RMS threshold values (tune during testing)
- Audio level bar visual design (number of bars, colors, animation)
- Thread communication pattern details (mpsc vs crossbeam)
- Exact cpal stream configuration code

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Existing Code
- `src-tauri/src/lib.rs` — Plugin setup, existing Tauri commands, ConfigStore access
- `src-tauri/Cargo.toml` — cpal and hound already listed as dependencies
- `src/App.tsx` — Hotkey registration (currently stub), event listening patterns
- `src/components/TrayPopup.tsx` — Status indicator to extend with recording state + audio bars

### Research
- `.planning/research/STACK.md` — cpal 0.16.0 usage patterns, hound encoding
- `.planning/research/ARCHITECTURE.md` — Audio data flow, Rust-owns-side-effects principle
- `.planning/research/PITFALLS.md` — Whisper hallucinations on silence, VAD requirement

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `cpal` already in Cargo.toml (Phase 2 added it for mic enumeration)
- `hound` already in Cargo.toml
- Hotkey registration stub in App.tsx ready to be replaced with real handler
- TrayPopup has status indicator area ready for recording state
- ConfigStore has `microphone_id` key from Phase 2

### Established Patterns
- Tauri events for backend→frontend communication (used in Phase 1/2)
- Tauri commands for frontend→backend requests
- ConfigStore for persistent settings
- German code comments

### Integration Points
- Replace hotkey stub handler with real recording start/stop logic
- Extend TrayPopup with audio level visualization
- Extend tray icon state machine (idle/recording/processing)
- WAV buffer output feeds into Phase 4 (STT pipeline)

</code_context>

<specifics>
## Specific Ideas

- Recording should feel instant — no perceptible delay between hotkey press and recording start
- VAD discard should be invisible to user except brief "Zu kurz" notification
- Audio bars should feel alive and responsive — not laggy or jumpy

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 03-recording-core*
*Context gathered: 2026-04-09*
