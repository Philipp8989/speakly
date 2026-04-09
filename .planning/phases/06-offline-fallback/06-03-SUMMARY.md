---
phase: 06-offline-fallback
plan: "03"
subsystem: frontend
tags: [stt, offline, download, ui, settings, tray]
dependency_graph:
  requires: []
  provides: [stt-engine-dropdown, download-progress-ui, windows-warning, tray-download-states]
  affects: [SettingsPage.tsx, TrayPopup.tsx]
tech_stack:
  added: []
  patterns: [tauri-event-listen, navigator-userAgent-platform-detection, react-discriminated-union-state]
key_files:
  created: []
  modified:
    - src/components/SettingsPage.tsx
    - src/components/TrayPopup.tsx
decisions:
  - "navigator.userAgent used for Windows detection instead of @tauri-apps/plugin-os (not installed)"
  - "Download managed in SettingsPage directly; TrayPopup only displays state passed from App.tsx"
metrics:
  duration: ~10min
  completed: "2026-04-09"
  tasks_completed: 2
  files_modified: 2
---

# Phase 06 Plan 03: Frontend STT-Engine UI Summary

**One-liner:** STT-Engine-Dropdown (cloud/local/auto) mit Download-Fortschrittsbalken in SettingsPage und 10-State AppState-Union in TrayPopup fuer offline Whisper-Modell-Verwaltung.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | SettingsPage STT-Engine-Sektion + Windows-Warning | 35903be | src/components/SettingsPage.tsx |
| 2 | TrayPopup download-needed + downloading States | 9ce3f76 | src/components/TrayPopup.tsx |

## What Was Built

### Task 1: SettingsPage.tsx
- STT-Engine `<select>` mit 3 Optionen: `cloud`, `local`, `auto` — Wert aus `settings.stt_mode`
- `handleSttModeChange`: speichert via `setSetting('stt_mode', ...)`, prueft `check_whisper_model_exists` (try/catch falls Command noch nicht registriert)
- `startModelDownload`: setzt downloadState, hoert `whisper_download_progress` + `whisper_download_complete` + `whisper_download_error` Events, startet `download_whisper_model` invoke
- `handleCancelDownload`: ruft `cancel_whisper_download` auf, setzt State auf idle
- Windows-Warning: einmalig pro Session beim ersten Wechsel zu `local` (D-15, D-16)
- Download-Fortschrittsbalken mit Prozent + MB-Anzeige + Abbrechen-Button (D-12, D-13)

### Task 2: TrayPopup.tsx
- `AppState` Union von 8 auf 10 States erweitert: `download-needed` und `downloading`
- `dotClass`: orange-400 fuer download-needed, blue-400 animate-pulse fuer downloading
- `statusText`: "Modell fehlt" / "Herunterladen... X%"
- `previewContent`: download-needed zeigt Hinweis + "Zu Einstellungen" Button; downloading zeigt Fortschrittsbalken mit percent + MB

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] @tauri-apps/plugin-os nicht installiert**
- **Found during:** Task 1
- **Issue:** Plan referenced `platform()` from `@tauri-apps/plugin-os` for Windows detection, but the package is not in package.json
- **Fix:** Used `navigator.userAgent.toLowerCase().includes('windows')` as fallback per plan's own HINWEIS
- **Files modified:** src/components/SettingsPage.tsx
- **Commit:** 35903be

## Known Stubs

None — all UI is wired to real Tauri commands (download_whisper_model, cancel_whisper_download, check_whisper_model_exists). Commands are guarded by try/catch since Plan 01/02 registers them on the Rust side.

## Threat Flags

None — no new network endpoints or auth paths introduced. stt_mode only accepts 3 hardcoded option values (T-06-09 accepted per threat register).

## Self-Check

- [x] src/components/SettingsPage.tsx modified and committed (35903be)
- [x] src/components/TrayPopup.tsx modified and committed (9ce3f76)
- [x] `npx tsc --noEmit` passes with no errors
- [x] grep shows stt_mode in SettingsPage.tsx, download-needed + downloading in TrayPopup.tsx

## Self-Check: PASSED
