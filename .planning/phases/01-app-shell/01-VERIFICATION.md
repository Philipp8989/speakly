---
phase: 01-app-shell
verified: 2026-04-09T10:00:00Z
status: human_needed
score: 4/5 must-haves verified
overrides_applied: 0
deferred:
  - truth: "Tray icon changes state during recording and processing (APP-01 dynamic status)"
    addressed_in: "Phase 3"
    evidence: "Phase 3 success criteria: 'Tray icon changes state during recording and processing'"
human_verification:
  - test: "Launch app with `pnpm tauri dev`, observe startup"
    expected: "No window appears on screen. Speakly icon appears in macOS menu bar or Windows system tray within 5 seconds."
    why_human: "Visual confirmation of tray-only startup cannot be verified programmatically — requires a running display session."
  - test: "Left-click the tray icon"
    expected: "A 300x200px popup window appears positioned near the tray. Shows: grey dot + 'Bereit' label at top, bordered preview area with placeholder text, gear icon (⚙) at bottom-right."
    why_human: "Visual rendering and window positioning require a running app with display."
  - test: "Click the close button (red dot on macOS, X on Windows) while popup is open"
    expected: "Popup closes. Tray icon remains visible. Process is still running (clicking tray icon reopens popup)."
    why_human: "Window lifecycle behavior (hide vs quit) must be observed in a running session."
  - test: "Right-click the tray icon, click 'Speakly beenden'"
    expected: "App exits completely — tray icon disappears from menu bar / system tray."
    why_human: "Tray menu rendering and quit behavior must be verified with a running app."
  - test: "After first launch, check `~/Library/Application Support/de.speakly.app/settings.json` (macOS)"
    expected: "File exists and contains `\"initialized\":true` and `\"hotkey\":\"CommandOrControl+Shift+Space\"`. File survives app restart with same content."
    why_human: "ConfigStore file creation and persistence requires an actual app run to write the file."
---

# Phase 1: App Shell Verification Report

**Phase Goal:** A runnable Tauri app lives in the system tray on macOS and Windows with correct lifecycle behavior
**Verified:** 2026-04-09T10:00:00Z
**Status:** human_needed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths (Roadmap Success Criteria)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | App launches and appears as a tray icon on macOS and Windows | ? HUMAN | `visible: false` in tauri.conf.json + `tray-icon` feature in Cargo.toml + TrayIconBuilder in tray.rs — architecture is correct; visual confirmation required |
| 2 | Clicking the tray icon opens a minimal popup/window | ? HUMAN | tray.rs left-click handler calls `move_window(Position::TrayCenter)` + `show()` + `set_focus()` — logic correct; behavior requires running app |
| 3 | Closing the window hides to tray instead of quitting | ? HUMAN | lib.rs `WindowEvent::CloseRequested` intercept: `api.prevent_close()` + `win_clone.hide()` — implementation present and substantive; must be observed |
| 4 | App consumes minimal resources when idle | ✓ VERIFIED | No background threads, no audio or network polling found in lib.rs, tray.rs, App.tsx, or TrayPopup.tsx. Event-driven only. |
| 5 | ConfigStore is initialized and persists a test key across restarts | ? HUMAN | lib.rs writes `hotkey`, `stt_mode`, `initialized` to `settings.json` on first run; App.tsx mirrors this on frontend. Persistence across restarts requires actual file system verification. |

**Score:** 1/5 truths mechanically verified; 4/5 have correct implementation evidence (human observation required for 4 of them)

### Deferred Items

Items not yet met but explicitly addressed in later milestone phases.

| # | Item | Addressed In | Evidence |
|---|------|-------------|----------|
| 1 | Tray icon shows idle/recording/processing status dynamically (APP-01 full scope) | Phase 3 | Phase 3 success criteria: "Tray icon changes state during recording and processing" |

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `src-tauri/Cargo.toml` | Rust deps including all three plugins | ✓ VERIFIED | Contains `tauri-plugin-store = "2"`, `tauri-plugin-positioner = { version = "2", features = ["tray-icon"] }`, `tauri-plugin-global-shortcut = "2"`, `serde_json = "1"` |
| `src-tauri/capabilities/default.json` | Plugin permissions | ✓ VERIFIED | Contains `store:default`, `positioner:default`, `global-shortcut:allow-register`, `allow-unregister`, `allow-is-registered`, `core:default` |
| `src-tauri/tauri.conf.json` | Window config with visible:false | ✓ VERIFIED | `visible: false`, `decorations: false`, `skipTaskbar: true`, `alwaysOnTop: true`, `iconAsTemplate: false`, `menuOnLeftClick: false` |
| `src-tauri/src/lib.rs` | Plugin wiring + close-to-tray intercept | ✓ VERIFIED | Registers all 3 plugins, delegates to `tray::setup_tray`, implements `CloseRequested` prevent_close + hide |
| `src-tauri/src/tray.rs` | TrayIconBuilder + positioned popup + quit menu | ✓ VERIFIED | TrayIconBuilder, Position::TrayCenter, `on_tray_event` forwarding, `app.exit(0)`, "Speakly beenden" menu item |
| `src/components/TrayPopup.tsx` | Popup UI with 3 elements, min 40 lines | ⚠️ NEAR-MISS | File has 37 lines (plan spec: min 40). All 3 UI elements present: grey dot + "Bereit", transcription preview, settings gear. Functionally complete — 3-line shortfall is cosmetic. |
| `src/App.tsx` | ConfigStore init + hotkey registration | ✓ VERIFIED | Imports plugin-store and plugin-global-shortcut, calls `initStore()` and `registerHotkey()` on mount, TrayPopup wired |
| `src/main.tsx` | React entry point with createRoot | ✓ VERIFIED | `ReactDOM.createRoot` present, renders `<App />` |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `src-tauri/tauri.conf.json` | `capabilities/default.json` | Tauri capabilities system | ✓ VERIFIED | `store:default` present in both config and capabilities |
| `src-tauri/Cargo.toml` | `tauri-plugin-positioner` | cargo dependency | ✓ VERIFIED | `tauri-plugin-positioner = { version = "2", features = ["tray-icon"] }` |
| `src-tauri/src/lib.rs` | `src-tauri/src/tray.rs` | `mod tray; tray::setup_tray(app)` | ✓ VERIFIED | `mod tray;` at line 4, `tray::setup_tray(app)?` at line 28 |
| `tray.rs on_tray_icon_event` | `tauri_plugin_positioner::on_tray_event` | positioner event forwarding | ✓ VERIFIED | `tauri_plugin_positioner::on_tray_event(tray.app_handle(), &event)` at line 32 |
| `tray.rs left click handler` | `window.move_window(Position::TrayCenter)` | tauri-plugin-positioner WindowExt | ✓ VERIFIED | `window.move_window(Position::TrayCenter)` at line 47 |
| `src/App.tsx` | `@tauri-apps/plugin-store` | `load('settings.json')` | ✓ VERIFIED | `import { load } from '@tauri-apps/plugin-store'` + used in `initStore()` |
| `src/App.tsx` | `@tauri-apps/plugin-global-shortcut` | `register('CommandOrControl+Shift+Space')` | ✓ VERIFIED | `import { register } from '@tauri-apps/plugin-global-shortcut'` + used in `registerHotkey()` |

### Data-Flow Trace (Level 4)

TrayPopup renders static UI only (no dynamic data from store in Phase 1). ConfigStore writes to disk — no frontend rendering of stored data in Phase 1. Level 4 trace not applicable to Phase 1 components (all rendering is static/structural by design).

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| Rust backend compiles without errors | `cargo check` (with `~/.cargo/bin` in PATH) | `Finished dev profile [unoptimized + debuginfo] target(s) in 31.43s` — zero errors | ✓ PASS |
| All three plugins in Cargo.toml | `grep "tauri-plugin-store\|tauri-plugin-positioner\|tauri-plugin-global-shortcut" Cargo.toml` | All three lines found with version "2" | ✓ PASS |
| prevent_close in lib.rs | `grep "prevent_close" src-tauri/src/lib.rs` | Line 36: `api.prevent_close();` | ✓ PASS |
| TrayCenter positioning in tray.rs | `grep "TrayCenter" src-tauri/src/tray.rs` | Line 47: `window.move_window(Position::TrayCenter)` | ✓ PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| APP-01 | 01-02, 01-03, 01-04 | System Tray Icon zeigt Status (idle/recording/processing) | PARTIAL | Tray icon exists and shows idle state (grey dot in TrayPopup). Dynamic recording/processing status deferred to Phase 3. Tray icon presence is verified; dynamic states are not yet implemented — correctly deferred. |
| APP-05 | 01-01, 01-04 | App laeuft auf macOS und Windows | ? HUMAN | Cargo.toml has no platform-specific exclusions; `cargo check` passes on macOS aarch64. Windows compilation not verified in this environment — requires CI or Windows machine. |
| APP-06 | 01-01, 01-02, 01-04 | App startet minimiert im System Tray mit minimalem Ressourcenverbrauch | ? HUMAN | `visible: false` + `skipTaskbar: true` in tauri.conf.json; no background threads/polling found. Visual confirmation of tray-only startup requires running app. |

No orphaned requirements: REQUIREMENTS.md maps only APP-01, APP-05, APP-06 to Phase 1, which exactly matches the plan declarations.

### Anti-Patterns Found

| File | Pattern | Severity | Impact |
|------|---------|----------|--------|
| `src/App.tsx` | `handleSettingsClick` logs only (console.log stub) | ℹ️ Info | Intentional Phase 1 stub — Phase 2 opens settings window. No user-visible data blocked. |
| `src/App.tsx` | Hotkey handler logs only | ℹ️ Info | Intentional Phase 1 stub — Phase 3 adds recording logic. Hotkey registration itself is functional. |
| `src/components/TrayPopup.tsx` | Transcription preview always empty | ℹ️ Info | Intentional Phase 1 stub — Phase 4 wires real transcription data. |
| `src/components/TrayPopup.tsx` | Status dot always grey | ℹ️ Info | Intentional Phase 1 stub — Phase 3 adds recording/processing states. |

No blockers found. All stubs are explicitly documented in 01-03-SUMMARY.md as intentional with clear phase assignments.

### Human Verification Required

#### 1. Tray-Only Startup

**Test:** Run `pnpm tauri dev` from the project root.
**Expected:** No window appears on screen. Speakly icon appears in the macOS menu bar or Windows system tray within 5 seconds.
**Why human:** Visual confirmation of tray-only startup cannot be verified without a running display session.

#### 2. Tray Click Opens Positioned Popup

**Test:** Left-click the tray icon.
**Expected:** A ~300x200px dark popup window appears positioned near the tray icon. Must show: grey dot + "Bereit" label (top), bordered box with italic placeholder text (center), gear icon ⚙ (bottom-right).
**Why human:** Window positioning and visual rendering require a running app.

#### 3. Close Hides to Tray (Does Not Quit)

**Test:** With popup open, click the close button (red dot on macOS, X on Windows).
**Expected:** Popup closes. Tray icon STILL VISIBLE in menu bar. Process still running (clicking tray icon reopens popup).
**Why human:** Window lifecycle behavior (hide vs quit) must be observed in a live session.

#### 4. Right-Click Quit Menu

**Test:** Right-click the tray icon, then click "Speakly beenden".
**Expected:** App exits completely — tray icon disappears.
**Why human:** Tray context menu rendering and quit signal must be observed in a running app.

#### 5. ConfigStore Persistence

**Test:** After first launch, check `~/Library/Application Support/de.speakly.app/settings.json` (macOS) or the Windows equivalent.
**Expected:** File exists containing `"initialized":true` and `"hotkey":"CommandOrControl+Shift+Space"`. After restarting the app, file content is unchanged.
**Why human:** ConfigStore file creation requires an actual app run to invoke the Tauri store plugin write path.

### Gaps Summary

No programmatic gaps found. All required code artifacts are present, substantive, and wired. `cargo check` passes with zero errors.

The phase cannot be marked `passed` because 4 of 5 roadmap success criteria require visual/behavioral confirmation that can only be provided by running the app on a device with a display. The implementation evidence for all 4 human-needed items is strong — the verification items are confirmatory, not investigatory.

One artifact (TrayPopup.tsx at 37 lines) is 3 lines below the `min_lines: 40` artifact spec but is functionally complete and not a stub.

APP-01 dynamic tray status (recording/processing states) is correctly deferred to Phase 3, which explicitly includes "Tray icon changes state during recording and processing" as a success criterion.

---

_Verified: 2026-04-09T10:00:00Z_
_Verifier: Claude (gsd-verifier)_
