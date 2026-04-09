---
phase: 01-app-shell
plan: "03"
subsystem: frontend-ui
tags: [react, tailwind, tauri-plugins, frontend, hotkey, store]
dependency_graph:
  requires: [01-01-PLAN.md]
  provides: [TrayPopup component, ConfigStore init, global hotkey stub]
  affects: [src/App.tsx, src/components/TrayPopup.tsx, src/index.css, src/main.tsx]
tech_stack:
  added: [tailwindcss@4.2.2, "@tailwindcss/vite@4.2.2"]
  patterns: [Tailwind CSS v4 @import syntax, Tauri plugin-store v2 load() API, plugin-global-shortcut register()]
key_files:
  created:
    - src/components/TrayPopup.tsx
    - src/index.css
  modified:
    - src/App.tsx
    - src/main.tsx
    - vite.config.ts
    - package.json
decisions:
  - "Tailwind CSS v4 requires @tailwindcss/vite plugin (not postcss) and @import 'tailwindcss' directive (not @tailwind base/components/utilities)"
  - "StoreOptions.defaults is required in plugin-store 2.4.2 — pass empty object {} alongside autoSave"
  - "TypeScript compiled cleanly on second attempt after StoreOptions fix"
metrics:
  duration: "~10min"
  completed: "2026-04-09"
  tasks_completed: 2
  files_changed: 6
---

# Phase 1 Plan 03: React Frontend — TrayPopup and App Wiring Summary

React frontend built: TrayPopup component with idle status indicator, transcription preview, and settings gear stub; App.tsx wired with Tailwind CSS v4, plugin-store init, and global hotkey registration.

## Tasks Completed

### Task 1: Create TrayPopup component
**Commit:** 120d610

Created `src/components/TrayPopup.tsx` (37 lines) with three D-02 elements:
- Grey status dot + "Bereit" label (idle state, becomes green/amber in Phase 3)
- Transcription preview area with placeholder text (filled in Phase 4)
- Settings gear button (⚙) calling `onSettingsClick` prop (Phase 2 stub)

Dark bg-gray-900 layout fills the 300x200px tray window via `h-full`.

### Task 2: Wire App.tsx with ConfigStore init and hotkey registration
**Commit:** 4a2a1e4

- App.tsx replaced: imports plugin-store + plugin-global-shortcut, initializes defaults on first run
- `initStore()`: loads settings.json, sets hotkey/stt_mode/initialized if not yet set
- `registerHotkey()`: registers CommandOrControl+Shift+Space with try/catch (T-03-02 mitigated)
- `index.css` created with `@import "tailwindcss"` (Tailwind v4 syntax)
- `vite.config.ts` updated with `@tailwindcss/vite` plugin
- `main.tsx` updated to import index.css

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed StoreOptions.defaults required field**
- **Found during:** Task 2 — TypeScript compilation (`pnpm tsc --noEmit`)
- **Issue:** `plugin-store` v2.4.2 `StoreOptions` type has `defaults` as a required field (not optional). The plan's interface example used `{ autoSave: true }` without `defaults`, causing TS2345 error.
- **Fix:** Changed `load('settings.json', { autoSave: true })` to `load('settings.json', { defaults: {}, autoSave: true })`
- **Files modified:** src/App.tsx
- **Commit:** included in 4a2a1e4

**2. [Rule 3 - Blocking] Tailwind CSS v4 not installed**
- **Found during:** Task 2 setup
- **Issue:** Tailwind CSS was not in package.json; plan assumed it was already installed
- **Fix:** Ran `pnpm add tailwindcss @tailwindcss/vite`; added `tailwindcss()` to vite.config.ts plugins
- **Files modified:** package.json, pnpm-lock.yaml, vite.config.ts
- **Commit:** included in 4a2a1e4

## TypeScript Compilation

Failed on first attempt (StoreOptions.defaults missing). Passed cleanly after fix. Zero errors on final `pnpm tsc --noEmit`.

## Tailwind CSS v4 Import Syntax

The working syntax for Tailwind CSS v4:
```css
@import "tailwindcss";
```
Not `@tailwind base; @tailwind components; @tailwind utilities;` (v3 syntax). The v4 Vite plugin (`@tailwindcss/vite`) replaces the PostCSS approach entirely.

## React 19 / Tauri Plugin Async Compatibility

No issues. `useEffect` with `.catch(console.error)` pattern works cleanly with React 19. StrictMode double-invocation in dev is acceptable per plan.

## Popup Appearance

300x200px dark popup (bg-gray-900):
- Top: grey dot + "Bereit" label
- Center: bordered box with italic placeholder text
- Bottom-right: ⚙ gear button

## Known Stubs

| Stub | File | Line | Reason |
|------|------|------|--------|
| `handleSettingsClick` logs to console only | src/App.tsx | ~49 | Phase 2 opens settings window |
| Hotkey handler logs only | src/App.tsx | ~30 | Phase 3 triggers audio recording |
| Transcription preview always empty | src/components/TrayPopup.tsx | ~19 | Phase 4 wires real transcription data |
| Status dot always grey | src/components/TrayPopup.tsx | ~13 | Phase 3 adds recording/processing states |

These stubs are intentional for Phase 1 and do not block the plan's goal (popup UI renders correctly).

## Threat Flags

None. No new network endpoints, auth paths, or file access patterns beyond what was planned.

## Self-Check: PASSED

- src/components/TrayPopup.tsx: FOUND
- src/App.tsx: FOUND (modified)
- src/index.css: FOUND
- Commit 120d610: FOUND
- Commit 4a2a1e4: FOUND
- `pnpm tsc --noEmit`: zero errors
