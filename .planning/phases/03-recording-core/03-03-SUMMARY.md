---
phase: 03-recording-core
plan: "03"
subsystem: frontend-ui
tags: [recording, ui, audio-level, tray-popup, tauri-events]
dependency_graph:
  requires:
    - "03-01"
    - "03-02"
  provides:
    - AudioLevelBars component
    - Recording state UI in TrayPopup
  affects:
    - src/components/TrayPopup.tsx
    - src/components/AudioLevelBars.tsx
tech_stack:
  added: []
  patterns:
    - "Tauri listen() in useEffect with cleanup array"
    - "Conditional rendering based on recording state"
    - "RMS clamp for safe UI rendering"
key_files:
  created:
    - src/components/AudioLevelBars.tsx
  modified:
    - src/components/TrayPopup.tsx
decisions:
  - "Event listeners moved into TrayPopup instead of App.tsx for colocation with UI state"
  - "RMS clamped in AudioLevelBars per threat model T-03-08"
  - "isProcessing set to true on recording_stopped+wav_ready as Phase 4 stub"
metrics:
  duration_minutes: 8
  completed: "2026-04-09T09:01:18Z"
  tasks_completed: 2
  files_modified: 2
---

# Phase 3 Plan 03: Recording UI — AudioLevelBars + TrayPopup State Summary

## One-liner

Animated 8-bar audio level display with rms-proportional heights, wired to TrayPopup via four Tauri events for idle/recording/processing states and discarded toast.

## What Was Built

### AudioLevelBars component (`src/components/AudioLevelBars.tsx`)

New purely presentational component rendering 8 vertical bars:
- Heights computed as `max(2, round(rms * 32 * multiplier[i]))` — creates waveform visual variation
- `active=true`: bg-red-500, animated via transition-all duration-75
- `active=false`: bg-gray-600, flat 4px dormant state
- rms clamped to 0..1 (T-03-08 mitigation)

### TrayPopup extended (`src/components/TrayPopup.tsx`)

Four Tauri event listeners wired via useEffect:

| Event | Handler |
|-------|---------|
| `recording_state_changed` | Sets `isRecording`, resets `rms` on stop |
| `audio_level` | Updates `rms` state for bar animation |
| `recording_discarded` | Sets `discardedMessage` ("Zu kurz" / "Zu leise"), clears after 2000ms |
| `recording_stopped` | Sets `isProcessing=true` when wav_ready (Phase 4 stub) |

Three visual states:
- **Idle**: gray dot + "Bereit" (gray text)
- **Recording**: `bg-red-500 animate-pulse` + "Aufnahme..." (red text) + AudioLevelBars
- **Processing**: `bg-yellow-500 animate-pulse` + "Verarbeite..." (yellow text)

All listeners cleaned up on component unmount via `return () => { unlisteners.forEach(ul => ul()); }`.

## Commits

| Task | Commit | Description |
|------|--------|-------------|
| Task 1: AudioLevelBars | `4a01991` | feat(03-03): create AudioLevelBars component |
| Task 2: TrayPopup | `3ae05dc` | feat(03-03): extend TrayPopup with recording state UI |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Security] Applied T-03-08 rms clamp in AudioLevelBars**
- **Found during:** Task 1 (threat model review)
- **Issue:** Plan's threat model assigned `mitigate` disposition to T-03-08 — rms out-of-range values from Tauri events could cause negative bar heights or overflow
- **Fix:** Added `Math.max(0, Math.min(1, rms))` before height calculation in AudioLevelBars
- **Files modified:** `src/components/AudioLevelBars.tsx`
- **Commit:** `4a01991`

**Note:** Pre-existing TypeScript error in `src/hooks/useSettings.ts` (missing `defaults` in StoreOptions) was present before this plan and is out of scope.

## Known Stubs

| Stub | File | Line | Reason |
|------|------|------|--------|
| `isProcessing` set on `recording_stopped` but never cleared | `src/components/TrayPopup.tsx` | ~47 | Phase 4 (transcription pipeline) will clear it when done |
| Transcript preview shows static placeholder text | `src/components/TrayPopup.tsx` | ~75 | Phase 4 will wire real transcript |

## Threat Flags

None. All mitigations from the plan's threat model applied.

## Self-Check: PASSED

- `src/components/AudioLevelBars.tsx` — exists, exports `AudioLevelBars`
- `src/components/TrayPopup.tsx` — contains all 4 event names, imports AudioLevelBars
- Commits `4a01991` and `3ae05dc` — verified in git log
- TypeScript: only pre-existing error in useSettings.ts (out of scope)
