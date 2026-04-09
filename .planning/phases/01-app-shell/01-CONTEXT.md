# Phase 1: App Shell - Context

**Gathered:** 2026-04-09
**Status:** Ready for planning

<domain>
## Phase Boundary

Deliver a runnable Tauri 2 desktop app that lives in the system tray on macOS and Windows. Correct lifecycle behavior: close hides to tray, quit via tray menu. ConfigStore persists settings. No audio, no network, no recording — just the shell.

</domain>

<decisions>
## Implementation Decisions

### Tray Interaction
- **D-01:** Clicking the tray icon opens a small popup window attached to the tray (not a separate floating window). This is the Voicy-style pattern — minimal, non-intrusive.
- **D-02:** The popup shows: recording state indicator (idle for now), last transcription preview area (empty for now), and a settings gear icon.

### Window Design
- **D-03:** Minimal status view in the popup. No elaborate UI in Phase 1 — just enough to confirm the app is alive and settings are accessible.
- **D-04:** Settings gear icon in popup opens a separate settings window (implemented in Phase 2).

### Close & Quit Behavior
- **D-05:** Closing the popup/window hides to tray on BOTH macOS and Windows. App stays running.
- **D-06:** Quit is available via tray right-click menu ("Quit Speakly") and via Cmd+Q (macOS) / Alt+F4 on the main window.
- **D-07:** On Windows, the X button on the window hides to tray (not quit). This is explicitly handled — Tauri default would quit.

### Default Hotkey
- **D-08:** Factory default hotkey is CommandOrControl+Shift+Space. Registered via tauri-plugin-global-shortcut. Not user-configurable until Phase 2.

### Claude's Discretion
- Tray icon design (idle state icon) — use a simple microphone outline icon
- Popup window dimensions — keep it small (~300x200px), adjust as needed
- React component structure for the popup

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Tauri Core
- `.planning/research/STACK.md` — Full technology stack with versions and rationale
- `.planning/research/ARCHITECTURE.md` — Component boundaries and IPC patterns

### Pitfalls
- `.planning/research/PITFALLS.md` — Critical pitfalls including Windows close-to-tray behavior and macOS tray icon bugs

No external specs — requirements fully captured in decisions above.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- None — greenfield project, no existing code

### Established Patterns
- None yet — this phase ESTABLISHES the patterns (Tauri IPC, React component structure, ConfigStore usage)

### Integration Points
- ConfigStore initialized here will be used by all subsequent phases for settings persistence
- Tray icon state machine (idle) will be extended in Phase 3 (recording/processing states)
- Popup window layout will be extended in Phase 2 (settings) and Phase 3 (waveform)

</code_context>

<specifics>
## Specific Ideas

- Follow Voicy's tray popup pattern — small, attached to tray, non-intrusive
- App should feel lightweight and native, not like a web app in a wrapper

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 01-app-shell*
*Context gathered: 2026-04-09*
