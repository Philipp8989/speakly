---
phase: 02-settings-onboarding
plan: "03"
subsystem: frontend-onboarding
tags: [onboarding, wizard, app-routing, settings-window, tauri]
dependency_graph:
  requires: [02-01]
  provides: [onboarding-wizard, app-url-routing, settings-window-management]
  affects: [src/App.tsx, src/components/onboarding/]
tech_stack:
  added: []
  patterns:
    - 4-step wizard with local step state (no wizard library)
    - URL-param routing for multi-window Tauri apps (?window=settings&mode=X)
    - Single-window guard via isOpeningSettings flag + WebviewWindow.getByLabel()
    - onboarding_complete written in step 3 (before Done screen) to avoid race on window close
key_files:
  created:
    - src/components/onboarding/OnboardingWizard.tsx
    - src/components/onboarding/StepWelcome.tsx
    - src/components/onboarding/StepApiKeys.tsx
    - src/components/onboarding/StepMicrophone.tsx
    - src/components/onboarding/StepDone.tsx
    - src/components/SettingsPage.tsx
  modified:
    - src/App.tsx
decisions:
  - "openUrl (not open) is the correct export from @tauri-apps/plugin-opener 2.x"
  - "StoreOptions.defaults is required — pass { defaults: {}, autoSave: true } for autoSave-only usage"
  - "SettingsPage stub created to unblock compile; will be replaced by plan 02-01 merge"
metrics:
  duration_minutes: 25
  completed_date: "2026-04-09"
  tasks_completed: 2
  tasks_total: 2
  files_created: 6
  files_modified: 1
---

# Phase 2 Plan 03: Onboarding Wizard + App.tsx Routing Summary

**One-liner:** 4-step onboarding wizard with mic/permission checks and App.tsx URL routing for multi-window Tauri settings management.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Build 4 onboarding step components + OnboardingWizard container | e17f275 | 6 files created |
| 2 | Wire App.tsx — openSettingsWindow + onboarding auto-launch on startup | 71a6bfd | src/App.tsx |

## What Was Built

**OnboardingWizard** (`src/components/onboarding/OnboardingWizard.tsx`): Container with progress bar, step navigation (goNext/goBack), renders StepWelcome/StepApiKeys/StepMicrophone/StepDone based on step index. Registers close-intercept to hide instead of terminate.

**StepWelcome**: Shows hotkey display, single Weiter button (no Zurueck on step 1).

**StepApiKeys**: Password-masked inputs for OpenAI + Anthropic keys. Saves to ConfigStore on Weiter. Skip option bypasses save. Implements D-04 (optional keys).

**StepMicrophone**: Calls `list_audio_input_devices` Tauri command for mic dropdown. Calls `check_macos_permissions` for microphone + accessibility status. Shows "Einstellungen oeffnen" deep-link buttons for denied/undetermined permissions. Writes `onboarding_complete=true` and `microphone_id` to store on Weiter (D-03, Pitfall 4).

**StepDone**: Info screen with hotkey reminder. "Los geht's" hides the window.

**App.tsx** rewired:
- `openSettingsWindow(mode)` with double-open guard (isOpeningSettings + getByLabel)
- `checkOnboarding()` reads `onboarding_complete`, auto-opens wizard with 500ms delay on first launch
- `getWindowMode()` reads `?window=settings&mode=X` URL params for multi-window routing
- `initStore()` extended with all Phase 2 store defaults

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] plugin-opener exports openUrl, not open**
- **Found during:** Task 1 TypeScript check
- **Issue:** Plan specified `import { open } from '@tauri-apps/plugin-opener'` but the package exports `openUrl` (not `open`) in version 2.x
- **Fix:** Changed all `open(...)` calls to `openUrl(...)` in StepMicrophone.tsx
- **Files modified:** src/components/onboarding/StepMicrophone.tsx
- **Commit:** e17f275

**2. [Rule 1 - Bug] StoreOptions.defaults is required field**
- **Found during:** Task 1 TypeScript check
- **Issue:** `load('settings.json', { autoSave: true })` fails type check — `defaults` is a required property of StoreOptions
- **Fix:** Changed to `{ defaults: {}, autoSave: true }` in StepApiKeys.tsx, StepMicrophone.tsx, and App.tsx
- **Files modified:** src/components/onboarding/StepApiKeys.tsx, src/components/onboarding/StepMicrophone.tsx, src/App.tsx
- **Commit:** e17f275

**3. [Rule 3 - Blocking] SettingsPage missing (parallel plan 02-01)**
- **Found during:** Task 2 — App.tsx imports SettingsPage which doesn't exist yet
- **Issue:** Plan 02-03 depends on Plan 02-01 (SettingsPage) which is being built by a parallel agent
- **Fix:** Created `src/components/SettingsPage.tsx` as minimal stub so TypeScript compiles. Will be replaced when plan 02-01 merges.
- **Files modified:** src/components/SettingsPage.tsx (created)
- **Commit:** e17f275

## Known Stubs

| File | Stub | Reason |
|------|------|--------|
| src/components/SettingsPage.tsx | Minimal placeholder component | Plan 02-01 (parallel agent) provides the real implementation; stub unblocks TypeScript compile |

## Threat Surface

All threat mitigations from the plan were implemented:
- **T-02-08** (URL param routing): `getWindowMode()` only activates settings mode for `window=settings`; any other value renders TrayPopup
- **T-02-09** (deep-link URLs): All URLs in StepMicrophone are hardcoded string constants — no user input
- **T-02-10** (API key state): Keys only in React state, written to store on Weiter only, never logged
- **T-02-11** (double-open): `isOpeningSettings` flag + `WebviewWindow.getByLabel('settings')` check

## Self-Check: PASSED

Files verified:
- FOUND: src/components/onboarding/OnboardingWizard.tsx
- FOUND: src/components/onboarding/StepWelcome.tsx
- FOUND: src/components/onboarding/StepApiKeys.tsx
- FOUND: src/components/onboarding/StepMicrophone.tsx
- FOUND: src/components/onboarding/StepDone.tsx
- FOUND: src/components/SettingsPage.tsx
- FOUND: src/App.tsx (modified)

Commits verified:
- e17f275: feat(02-03): build 4-step onboarding wizard components
- 71a6bfd: feat(02-03): wire App.tsx with openSettingsWindow, onboarding check, URL routing

TypeScript: compiles without errors (tsc --noEmit exit 0)
