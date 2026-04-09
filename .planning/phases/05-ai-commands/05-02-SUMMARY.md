---
phase: 05-ai-commands
plan: "02"
subsystem: ai-ui
tags: [react, typescript, tauri, ai-commands, state-machine]
dependency_graph:
  requires: [05-01]
  provides: [command-select UI, AI command flow, inject_raw_text]
  affects: [src/components/TrayPopup.tsx, src/App.tsx, src-tauri/src/lib.rs]
tech_stack:
  added: [inject_raw_text Tauri command, useRef for timer management]
  patterns: [8-state-machine React UI, async invoke with error fallback, auto-dismiss timer]
key_files:
  created: []
  modified:
    - src/components/TrayPopup.tsx
    - src/App.tsx
    - src-tauri/src/lib.rs
decisions:
  - inject_raw_text added to lib.rs in Plan 02 (not Plan 01) to avoid wave conflict
  - handleCommandSelect is async — awaits inject_raw_text after reformulated state shown
  - autoInsertTimerRef cleared on hotkey press via recording_state_changed to implement D-16 cancel
metrics:
  duration: "~20 minutes"
  completed: "2026-04-09"
  tasks_completed: 2
  files_created: 0
  files_modified: 3
---

# Phase 05 Plan 02: AI Command UI and Flow Wiring Summary

8-state TrayPopup with pill-button command row, wired AI command flow from transcription complete through Claude API call to final paste, with raw-insert fallback, auto-dismiss timer, and hotkey cancel.

## What Was Built

- `src/components/TrayPopup.tsx`: Extended to 8-state machine adding `command-select` (transcript + 6 pill buttons + "Einfuegen"), `reformulating` (spinner + original text), `reformulated` (result in green). Buttons disabled with tooltip when no API key.
- `src/App.tsx`: Full AI command flow — `handleInsertRaw`, `handleCommandSelect`, `autoInsertTimerRef`, 10s auto-dismiss, hotkey-cancel in `recording_state_changed`, `onCommandSelect`/`onInsertRaw` wired to TrayPopup, `transcription_state_changed` done branch now sets `command-select` with `hasApiKey` from ConfigStore.
- `src-tauri/src/lib.rs`: Added `inject_raw_text` Tauri command (wraps `inject::inject_text`), registered in invoke_handler.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Extend AppState and TrayPopup with AI command states and UI | 37e024e | src/components/TrayPopup.tsx |
| 2 | Wire AI command flow in App.tsx + inject_raw_text in lib.rs | 26a9a46 | src/App.tsx, src-tauri/src/lib.rs |

## Verification

- `npx tsc --noEmit` — exits 0, zero errors (both tasks)
- `cargo build` — exits 0, 3 pre-existing dead_code warnings only, no new errors
- TrayPopup.tsx contains `command-select`, `reformulating`, `reformulated` in AppState type
- App.tsx contains `invoke('apply_ai_command'` and `invoke('inject_raw_text'`
- `inject_raw_text` registered in lib.rs invoke_handler at line 279

## Decisions Made

| Decision | Rationale |
|----------|-----------|
| `inject_raw_text` added in Plan 02, not Plan 01 | Plan 01 already complete; wave ordering ensures no conflict; plan 02 files_modified can include lib.rs |
| `handleCommandSelect` awaits `inject_raw_text` | Ensures inject completes before idle transition — avoids race where state flips before paste |
| Auto-dismiss clears timer ref before setting idle | Prevents double-fire if user clicks "Einfuegen" after timer already expired |
| D-16 cancel via `recording_state_changed` | Hotkey press triggers `recording_state_changed` with `recording: true` — cheapest hook point, no new Rust event needed |

## Deviations from Plan

### Auto-fixed Issues

None — plan executed exactly as written. The `inject_raw_text` addition to lib.rs was explicitly called out in the plan action as the correct approach.

## Known Stubs

None. All 6 command buttons are wired to `onCommandSelect` which calls `apply_ai_command`. The `hasApiKey` check reads live from ConfigStore.

## Threat Flags

None — all threat mitigations from plan threat_model applied:
- T-05-05: `commandId` derived from `AI_COMMANDS` const — clicking a button passes `cmd.id` from the fixed const array, never from free-text user input
- T-05-06: auto-dismiss timer fires `handleInsertRaw` which is safe (calls inject_raw_text)
- T-05-07: error messages in TrayPopup come from Rust's fixed German error strings, no key material echoed

## Self-Check: PASSED

- `src/components/TrayPopup.tsx` modified: FOUND
- `src/App.tsx` modified: FOUND
- `src-tauri/src/lib.rs` modified with inject_raw_text: FOUND
- Commit 37e024e (TrayPopup): FOUND
- Commit 26a9a46 (App.tsx + lib.rs): FOUND
