# Phase 1: App Shell - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-09
**Phase:** 01-app-shell
**Areas discussed:** Tray Popup, Window Design, Close Behavior, Default Hotkey
**Mode:** auto (all recommended defaults selected)

---

## Tray Popup

| Option | Description | Selected |
|--------|-------------|----------|
| Small popup attached to tray | Voicy-style, minimal, non-intrusive | ✓ |
| Separate floating window | Independent window, can be moved/resized | |
| Both (popup + main window) | Popup for quick access, window for details | |

**User's choice:** [auto] Small popup attached to tray (recommended default)
**Notes:** Standard pattern for dictation apps. Keeps the app unobtrusive.

---

## Window Design

| Option | Description | Selected |
|--------|-------------|----------|
| Minimal status view | Recording state, last transcription, settings gear | ✓ |
| Feature-rich dashboard | Full controls, history, settings inline | |
| Bare minimum (icon only) | Just tray icon, no popup window | |

**User's choice:** [auto] Minimal status view (recommended default)
**Notes:** Phase 1 is just the shell — keep it simple. More UI comes in later phases.

---

## Close Behavior

| Option | Description | Selected |
|--------|-------------|----------|
| Hide to tray on both platforms | Close = hide, Quit via tray menu/Cmd+Q | ✓ |
| Hide on macOS, quit on Windows | Follow each platform's convention | |
| Always quit on close | No tray persistence | |

**User's choice:** [auto] Hide to tray on both platforms (recommended default)
**Notes:** Tray apps should stay running. Quit explicitly via menu.

---

## Default Hotkey

| Option | Description | Selected |
|--------|-------------|----------|
| CommandOrControl+Shift+Space | Low conflict, easy to reach | ✓ |
| CommandOrControl+Shift+D | D for dictate, more memorable | |
| F5 | Single key, fast but conflict-prone | |

**User's choice:** [auto] CommandOrControl+Shift+Space (recommended default)
**Notes:** Matches stack research recommendation. Configurable in Phase 2.

---

## Claude's Discretion

- Tray icon design (idle state)
- Popup window dimensions
- React component structure

## Deferred Ideas

None
