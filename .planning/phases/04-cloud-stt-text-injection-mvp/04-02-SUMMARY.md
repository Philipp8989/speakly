---
phase: 04-cloud-stt-text-injection-mvp
plan: 02
subsystem: frontend
tags: [ui, state-machine, events, settings, tauri]
dependency_graph:
  requires: [04-01]
  provides: [frontend-stt-ui, language-settings]
  affects: [src/App.tsx, src/components/TrayPopup.tsx, src/components/SettingsPage.tsx]
tech_stack:
  added: []
  patterns: [tauri-event-listen, react-state-machine, tauri-invoke]
key_files:
  created: []
  modified:
    - src/components/TrayPopup.tsx
    - src/App.tsx
    - src/components/SettingsPage.tsx
    - src/hooks/useSettings.ts
decisions:
  - AppState type exported from TrayPopup.tsx as source of truth for all 5 states
  - useSettings.ts defaults property bug fixed inline (pre-existing TS error)
metrics:
  duration: ~15min
  completed: 2026-04-09T09:29:42Z
  tasks_completed: 2
  files_modified: 4
---

# Phase 04 Plan 02: Frontend STT UI — State Machine, Event Wiring, Language Dropdown Summary

**One-liner:** React 5-state machine (idle|recording|processing|transcript|error) wired to Tauri backend events with transcribe_and_inject invocation and language dropdown in Settings.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Extend TrayPopup + wire App.tsx event listeners | 176e3a0 | TrayPopup.tsx, App.tsx, useSettings.ts |
| 2 | Add language dropdown to SettingsPage (APP-03) | 90a6c40 | SettingsPage.tsx |

## What Was Built

### TrayPopup.tsx — 5-State Machine

Replaced the Phase 1 minimal stub with a full state-driven component:

- `idle`: gray dot, "Bereit", italic placeholder text
- `recording`: red pulsing dot, "Aufnahme..."
- `processing`: yellow pulsing dot, "Verarbeite...", animated dots (D-19)
- `transcript`: green dot, "Eingefuegt", transcript preview truncated to 120 chars (D-21)
- `error`: red static dot, "Fehler", error message in red (D-22)

`AppState` type exported from the component as the contract between App.tsx and TrayPopup.

### App.tsx — Event Listeners

Extended the existing useEffect with 4 event listeners:

1. `recording_state_changed` — sets `recording` or `idle` state
2. `recording_discarded` — logs only, no state change (no STT call per D-24)
3. `recording_stopped` — invokes `transcribe_and_inject` only when `wav_ready=true` (D-24 guard)
4. `transcription_state_changed` — drives processing/transcript/error states with 2s/5s auto-dismiss timers (D-21, D-22)

### SettingsPage.tsx — Language Dropdown

Added a "Sprache" section below API keys with a `<select>` dropdown:
- 15 options: `auto` (Automatisch erkennen) + 14 ISO 639-1 codes with German labels
- Reads current value from `useSettings` hook (`settings.language`)
- Writes on change via `setSetting('language', value)`
- Default `auto` maps to Whisper auto-detection (D-07, D-08, D-09)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed missing `defaults` property in useSettings.ts load() call**
- **Found during:** Task 1 TypeScript verification
- **Issue:** `load('settings.json', { autoSave: true })` was missing the required `defaults` property, causing a TS2345 type error
- **Fix:** Added `defaults: {}` to the options object to match `StoreOptions` interface
- **Files modified:** `src/hooks/useSettings.ts`
- **Commit:** 176e3a0

## Threat Surface Scan

T-04-09 (error message disclosure): Error messages from Rust are rendered as-is in TrayPopup. Per plan, Rust already sanitises to German user-facing strings before emitting — frontend renders exactly what it receives without transformation. No raw HTTP bodies forwarded. Mitigation satisfied.

No new threat surface introduced beyond the plan's threat model.

## Known Stubs

None — all states are wired to real backend events. No placeholder data flows to UI rendering.

## Self-Check

- [x] `src/components/TrayPopup.tsx` exists with "Verarbeite", "processing", "transcript" content
- [x] `src/App.tsx` contains "recording_stopped", "transcribe_and_inject", "wav_ready", "transcription_state_changed", "2000", "5000"
- [x] `src/components/SettingsPage.tsx` contains "language" (4+ matches), "auto", "Automatisch", "select"
- [x] `npx tsc --noEmit` exits 0
- [x] `npm run build` exits 0
- [x] Commits 176e3a0, 90a6c40 exist

## Self-Check: PASSED
