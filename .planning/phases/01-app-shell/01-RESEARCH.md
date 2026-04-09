# Phase 1: App Shell - Research

**Researched:** 2026-04-09
**Domain:** Tauri 2 system tray, window lifecycle, ConfigStore, global hotkey registration
**Confidence:** HIGH (all core decisions verified against official Tauri v2 docs)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** Clicking the tray icon opens a small popup window attached to the tray (not a separate floating window). Voicy-style pattern — minimal, non-intrusive.
- **D-02:** Popup shows: recording state indicator (idle), last transcription preview area (empty), settings gear icon.
- **D-03:** Minimal status view in popup. No elaborate UI in Phase 1 — just enough to confirm the app is alive and settings are accessible.
- **D-04:** Settings gear icon in popup opens a separate settings window (implemented in Phase 2).
- **D-05:** Closing the popup/window hides to tray on BOTH macOS and Windows. App stays running.
- **D-06:** Quit available via tray right-click menu ("Quit Speakly") and via Cmd+Q (macOS) / Alt+F4 on the main window.
- **D-07:** On Windows, the X button hides to tray (not quit). Tauri default would quit — must be explicitly overridden.
- **D-08:** Factory default hotkey is CommandOrControl+Shift+Space. Registered via tauri-plugin-global-shortcut. Not user-configurable until Phase 2.

### Claude's Discretion

- Tray icon design (idle state) — use a simple microphone outline icon
- Popup window dimensions — keep it small (~300x200px), adjust as needed
- React component structure for the popup

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope.

</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| APP-01 | System Tray Icon shows status (idle/recording/processing) | Tauri `tray-icon` feature (built-in) supports dynamic icon swap at runtime |
| APP-05 | App runs on macOS and Windows | Tauri 2.10.3 is first-class macOS + Windows; all chosen plugins confirmed cross-platform |
| APP-06 | App starts minimized in System Tray with minimal resource use | `visible: false` on startup window + tray-only initialization pattern; no audio/network until Phase 3 |

</phase_requirements>

---

## Summary

Phase 1 delivers the runnable shell: a Tauri 2 app that lives in the system tray, opens a small positioned popup on tray-click, hides to tray on window close, quits via tray menu, and persists a test config key via plugin-store. No audio, no network.

The three non-trivial implementation points are: (1) positioning the popup window relative to the tray icon using `tauri-plugin-positioner`, (2) intercepting the Windows close event to hide rather than quit, and (3) wiring `tauri-plugin-global-shortcut` to register the default hotkey without triggering audio — hotkey is registered here so Phase 3 can just call `toggle_recording` without re-registration.

**Primary recommendation:** Scaffold with `create-tauri-app --template react-ts`, add positioner + store + global-shortcut plugins, implement close-to-tray intercept on Windows, and validate on both platforms before declaring Phase 1 done.

---

## Project Constraints (from CLAUDE.md)

| Directive | Constraint |
|-----------|-----------|
| Tech stack | Tauri (Rust + Web frontend) — locked, no Electron |
| STT | OpenAI Whisper API (Cloud) + whisper.cpp (local) — out of scope for Phase 1 |
| AI | Claude API — out of scope for Phase 1 |
| Platform | Must work on macOS AND Windows — no platform-specific hacks |
| Budget | No running costs except API usage — user brings own keys |
| GSD workflow | All file edits must go through GSD commands (`/gsd-execute-phase`) |
| Code comments | In German |
| Functions | Small and focused, no unnecessary abstractions |

---

## Standard Stack

### Core (Phase 1 scope)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Tauri | 2.10.3 | Desktop shell + tray | Locked project choice. `tray-icon` feature built-in, no extra dependency. [VERIFIED: STACK.md] |
| React | 19.x | Popup UI | Locked project choice. Standard Tauri template. [VERIFIED: STACK.md] |
| TypeScript | 5.x | Type safety | Standard Tauri frontend. [VERIFIED: STACK.md] |
| Tailwind CSS | 4.x | Styling | Locked project choice. [VERIFIED: STACK.md] |
| tauri-plugin-store | 2.x | ConfigStore persistence | Official Tauri plugin for key-value settings. Auto-saves on write. [VERIFIED: v2.tauri.app/plugin/store/] |
| tauri-plugin-global-shortcut | 2.x | Register default hotkey | Official Tauri plugin. Register here in Phase 1 (no-op handler), extend in Phase 3. [VERIFIED: STACK.md] |
| tauri-plugin-positioner | 2.x | Position popup near tray icon | Official Tauri plugin. Port of electron-positioner. Required for D-01 (attached popup). [VERIFIED: v2.tauri.app/plugin/positioner/] |

### Installation

```bash
# Scaffold project
pnpm create tauri-app speakly --template react-ts

# Tauri plugins (from project root)
npm run tauri add store
npm run tauri add positioner
npm run tauri add global-shortcut

# Or manually:
# src-tauri/Cargo.toml:
# tauri-plugin-store = "2"
# tauri-plugin-positioner = { version = "2", features = ["tray-icon"] }
# tauri-plugin-global-shortcut = "2"

# Frontend
pnpm add @tauri-apps/plugin-store
pnpm add @tauri-apps/plugin-positioner
pnpm add @tauri-apps/plugin-global-shortcut
```

---

## Architecture Patterns

### Recommended Project Structure

```
speakly/
├── src/
│   ├── components/
│   │   └── TrayPopup.tsx       # Hauptansicht: Status + Vorschau + Settings-Icon
│   ├── App.tsx                 # Root-Komponente, Event-Listener
│   ├── main.tsx                # Vite entry
│   └── index.css               # Tailwind imports
├── src-tauri/
│   ├── src/
│   │   ├── lib.rs              # Tauri setup: tray, plugins, commands
│   │   ├── tray.rs             # TrayIconBuilder + Lifecycle
│   │   └── commands.rs         # Tauri commands (get_settings, quit_app)
│   ├── icons/
│   │   ├── icon.png            # Tray icon (macOS PNG required)
│   │   ├── icon.ico            # Tray icon (Windows ICO required)
│   │   └── 32x32.png
│   ├── capabilities/
│   │   └── default.json        # Permissions: store:default, positioner:default, global-shortcut:default
│   └── tauri.conf.json
└── package.json
```

### Pattern 1: Tray-Only Startup (no visible window at launch)

**What:** App starts without showing any window. Tray icon appears. Popup window is created but hidden.

**tauri.conf.json:**
```json
{
  "app": {
    "windows": [
      {
        "label": "main",
        "title": "Speakly",
        "width": 300,
        "height": 200,
        "visible": false,
        "decorations": false,
        "resizable": false,
        "alwaysOnTop": true,
        "skipTaskbar": true
      }
    ],
    "trayIcon": {
      "iconPath": "icons/icon.png",
      "iconAsTemplate": false,
      "menuOnLeftClick": false
    }
  }
}
```

**Key settings:**
- `visible: false` — window exists but not shown on startup [ASSUMED — standard Tauri pattern, verify against schema.tauri.app/config/2]
- `decorations: false` — no title bar for popup style [VERIFIED: Tauri docs mention decorations config]
- `skipTaskbar: true` — popup doesn't appear in Windows taskbar
- `iconAsTemplate: false` — workaround for macOS tray icon bug #13770 [VERIFIED: STACK.md]
- `menuOnLeftClick: false` — we handle left click manually to show the popup window

### Pattern 2: Tray Click → Show Positioned Popup

**What:** Left click on tray icon shows the popup window positioned near the tray. Click again or click away hides it.

```rust
// Source: v2.tauri.app/plugin/positioner/ + v2.tauri.app/learn/system-tray/
// src-tauri/src/lib.rs
use tauri::tray::{TrayIconBuilder, TrayIconEvent, MouseButton, MouseButtonState};
use tauri_plugin_positioner::{WindowExt, Position};

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            let tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .on_tray_icon_event(|tray, event| {
                    // Positioner braucht tray-Events für korrekte Positionierung
                    tauri_plugin_positioner::on_tray_event(tray.app_handle(), &event);

                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            if window.is_visible().unwrap_or(false) {
                                let _ = window.hide();
                            } else {
                                let _ = window.move_window(Position::TrayCenter);
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                })
                .build(app)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Fehler beim Starten der Tauri-Anwendung")
}
```

### Pattern 3: Close Hides to Tray (Windows fix — D-07)

**What:** Intercept `CloseRequested` to hide the window instead of quitting. Critical for Windows where the default behavior terminates the process.

```rust
// Source: github.com/tauri-apps/tauri/discussions/11489
// Wird in setup() auf das Fenster-Objekt angewendet
use tauri::{WindowEvent, Manager};

let window = app.get_webview_window("main").unwrap();
let win_clone = window.clone();
window.on_window_event(move |event| {
    if let WindowEvent::CloseRequested { api, .. } = event {
        api.prevent_close();
        let _ = win_clone.hide();
    }
});
```

**Why this is critical:** On Windows, closing the window kills the Tauri process entirely. This means the global hotkey stops working — exactly the opposite of what a tray app should do. This must be in place from the first working build.

### Pattern 4: Tray Right-Click Menu with Quit

```rust
// Source: v2.tauri.app/learn/system-tray/
use tauri::menu::{Menu, MenuItem};

let quit = MenuItem::with_id(app, "quit", "Speakly beenden", true, None::<&str>)?;
let menu = Menu::with_items(app, &[&quit])?;

TrayIconBuilder::new()
    .menu(&menu)
    .on_menu_event(|app, event| match event.id.as_ref() {
        "quit" => app.exit(0),
        _ => {}
    })
    // ...
```

### Pattern 5: ConfigStore Init + Persist Test Key

```typescript
// Source: v2.tauri.app/plugin/store/
// src/main.tsx oder App.tsx
import { load } from '@tauri-apps/plugin-store';

const store = await load('settings.json', { autoSave: true });

// Testwert beim ersten Start setzen
const initialized = await store.get<boolean>('initialized');
if (!initialized) {
    await store.set('initialized', true);
    await store.set('hotkey', 'CommandOrControl+Shift+Space');
    await store.set('stt_mode', 'cloud');
}
```

```rust
// Rust-seitig: StoreExt für direkten Zugriff
// Source: v2.tauri.app/plugin/store/
use tauri_plugin_store::StoreExt;
use serde_json::json;

// In setup():
let store = app.store("settings.json")?;
if store.get("initialized").is_none() {
    store.set("hotkey", json!("CommandOrControl+Shift+Space"));
    store.set("stt_mode", json!("cloud"));
    store.set("initialized", json!(true));
}
```

### Pattern 6: Global Shortcut Registration (Phase 1 stub)

Register the hotkey in Phase 1 with a no-op handler. Phase 3 will update the handler to trigger recording.

```typescript
// Source: v2.tauri.app/plugin/global-shortcut/
import { register } from '@tauri-apps/plugin-global-shortcut';

// In App.tsx useEffect:
await register('CommandOrControl+Shift+Space', (shortcut) => {
    // Phase 1: Noch kein Handler — nur Registrierung bestätigen
    console.log('Hotkey registriert:', shortcut);
});
```

**Note on macOS:** `tauri-plugin-global-shortcut` does NOT require Accessibility permission to register global shortcuts — it uses the CGEventTap API which is separate from the Accessibility permission needed by `enigo` for key simulation. The Accessibility permission concern only applies to Phase 3+ (paste simulation). [ASSUMED — based on tauri-plugin-global-shortcut docs and GitHub discussions; verify if registration silently fails on clean macOS]

### Anti-Patterns to Avoid

- **Don't use `menuOnLeftClick: true`:** This shows a dropdown menu on left click, not a popup window. Use `false` and handle the click event manually to show/hide the window.
- **Don't skip the close-to-tray intercept:** The Tauri default on Windows terminates the process on window close. Without `prevent_close()`, every window close kills the hotkey listener.
- **Don't set `iconAsTemplate: true` on macOS:** Known tray icon visibility bug (#13770). Always use `iconAsTemplate: false` with an explicit PNG.
- **Don't use `rdev` for hotkeys:** Known focus-loss bug with Tauri windows. Use `tauri-plugin-global-shortcut` only.
- **Don't create the popup as a separate process/window from scratch on each click:** Create it once at startup (`visible: false`), then show/hide. Creating a new window on each click causes flicker and inconsistent positioning.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Tray-relative window positioning | Custom coordinate calculation | `tauri-plugin-positioner` with `Position::TrayCenter` | Different screen densities, multi-monitor, tray position varies per OS; positioner handles all edge cases |
| Settings persistence | Manual JSON file read/write | `tauri-plugin-store` | File locking, atomic writes, async API, auto-save — all handled |
| Global hotkey conflict detection | OS API calls | `tauri-plugin-global-shortcut` return value / error handling | Plugin wraps platform APIs; just check for errors on registration |
| Window positioning on macOS notch | Manual offset math | `tauri-plugin-positioner` `TrayCenter` position | Plugin accounts for macOS menu bar height and notch |

---

## Common Pitfalls

### Pitfall 1: Windows Close Kills the Process (CRITICAL — D-07)

**What goes wrong:** User closes the popup window on Windows → entire process exits → global hotkey stops working.

**Why it happens:** Tauri default on Windows: when the last window closes, the process exits. macOS keeps the process alive.

**How to avoid:** Intercept `WindowEvent::CloseRequested` and call `api.prevent_close()` + `window.hide()`. Must be done in `setup()` BEFORE the app runs. See Pattern 3.

**Warning signs:** App disappears from tray after closing window on Windows.

---

### Pitfall 2: Tray Icon Not Visible on macOS (Known Bug #13770)

**What goes wrong:** Tray icon occasionally doesn't appear after launch on macOS with certain Tauri 2.x builds.

**Why it happens:** Bug in `iconAsTemplate: true` handling.

**How to avoid:** Set `iconAsTemplate: false` in `tauri.conf.json` and provide explicit PNG file. [VERIFIED: STACK.md from prior research]

---

### Pitfall 3: Popup Positioned Off-Screen on Multi-Monitor

**What goes wrong:** Window appears at wrong coordinates if the tray is on a secondary monitor or the user has unusual display scaling.

**Why it happens:** Manual coordinate calculation doesn't account for all display configurations.

**How to avoid:** Use `tauri-plugin-positioner` with `Position::TrayCenter`. Do NOT calculate window position manually.

---

### Pitfall 4: Global Hotkey Registration Silently Fails

**What goes wrong:** `CommandOrControl+Shift+Space` is claimed by another app (Slack, system shortcuts). Registration appears to succeed but hotkey doesn't fire.

**Why it happens:** OS gives exclusive ownership of hotkey combinations. Conflict = silent no-op on some platforms.

**How to avoid:** Always wrap registration in error handling. Phase 1 should log a clear warning if registration fails. Phase 2 will add the settings UI to change the hotkey. [VERIFIED: PITFALLS.md]

---

### Pitfall 5: Store Permissions Not Configured

**What goes wrong:** `tauri-plugin-store` calls fail at runtime with permission errors even though the plugin is initialized.

**Why it happens:** Tauri 2 has a capabilities/permissions system. Plugins require explicit permission grants.

**How to avoid:** Add `"store:default"`, `"positioner:default"`, `"global-shortcut:allow-register"` to `src-tauri/capabilities/default.json`. [VERIFIED: v2.tauri.app/plugin/store/]

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Node.js | Frontend build, pnpm | Yes | v22.22.0 | — |
| pnpm | Package management | Yes | 10.33.0 | npm (available) |
| Rust / cargo | Tauri backend compilation | No | — | Must install via rustup |
| Xcode CLI Tools | macOS build target | Yes | Installed | — |
| create-tauri-app | Project scaffolding | No | — | `cargo install create-tauri-app` after Rust install |

**Missing dependencies with no fallback:**

- **Rust / cargo** — Required to compile the Tauri backend. Must be installed before any Tauri work. Install via: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- **create-tauri-app** — After Rust is installed: `cargo install create-tauri-app` OR use `pnpm create tauri-app` (npm-based scaffold, no prior Rust required)

**Note:** `pnpm create tauri-app` works without a pre-installed Rust toolchain on first run — it will prompt to install Rust if not detected. This is the recommended entry point. [ASSUMED — based on Tauri docs; verify before Wave 0]

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | None established yet — greenfield project |
| Config file | None — Wave 0 creates it |
| Quick run command | `pnpm tauri dev` (manual verification) |
| Full suite command | N/A — Phase 1 is UI/shell; automated tests deferred to Phase 3+ |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| APP-01 | Tray icon visible after launch | Manual smoke | `pnpm tauri dev` → visually inspect tray | No — manual |
| APP-01 | Clicking tray opens popup window | Manual smoke | `pnpm tauri dev` → click tray icon | No — manual |
| APP-05 | App builds and runs on macOS | Manual smoke | `pnpm tauri build` (macOS runner) | No — manual |
| APP-05 | App builds and runs on Windows | Manual smoke | `pnpm tauri build` (Windows runner) | No — CI only |
| APP-06 | App starts in tray, no window shown | Manual smoke | `pnpm tauri dev` → check no window spawns | No — manual |
| APP-06 | Closing window hides to tray, not quit | Manual smoke | Open popup → click X → verify tray still present | No — manual |
| ConfigStore | Test key persists across restarts | Manual smoke | Restart app → check store file contains key | No — manual |

**Sampling rate:** All Phase 1 validations are manual smoke tests. Automated testing infrastructure (Vitest for frontend, Rust `#[test]`) is out of scope for Phase 1 but should be scaffolded in Wave 0 as empty test files.

### Wave 0 Gaps

- [ ] No test files exist — project not yet scaffolded
- [ ] Rust toolchain not installed — install before any work begins
- [ ] `src-tauri/capabilities/default.json` — needs plugin permissions added during scaffold

*(Note: Phase 1 is primarily a build/run/click validation phase. Automated unit tests will be added starting Phase 3 when business logic exists.)*

---

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | No | No auth in Phase 1 |
| V3 Session Management | No | No sessions |
| V4 Access Control | No | Single-user desktop app |
| V5 Input Validation | Partial | Store keys are internal only in Phase 1; no user input |
| V6 Cryptography | No | Settings stored as plain JSON (API keys deferred to Phase 2) |

**Phase 1 security note:** `tauri-plugin-store` writes plain JSON to disk. API keys are NOT stored in Phase 1 (only test keys like `initialized: true` and `hotkey: "CommandOrControl+Shift+Space"`). The security risk of plaintext storage is acknowledged and intentionally deferred to Phase 2 (OS keychain assessment). [VERIFIED: STATE.md — "API key storage security: tauri-plugin-store writes plain JSON — assess OS keychain option in Phase 2"]

### Known Threat Patterns

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Tray icon spoofing (fake app in tray) | Spoofing | Not mitigable at app level; user education only |
| Store file tampering | Tampering | Acceptable for Phase 1 non-sensitive data; Phase 2 adds integrity check if needed |

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `visible: false` is a valid `tauri.conf.json` window property in v2.10.3 | Pattern 1 | Window might be visible on startup; check schema.tauri.app/config/2 |
| A2 | `pnpm create tauri-app` installs Rust automatically if not present | Environment Availability | Wave 0 task order wrong; may need manual Rust install first |
| A3 | `tauri-plugin-global-shortcut` does NOT require macOS Accessibility permission | Pattern 6 | Registration succeeds but hotkey silently ignored on macOS; need explicit verification on clean macOS |
| A4 | `Position::TrayCenter` positions popup correctly on macOS menu bar (with notch) | Pattern 2 | Popup appears offset on newer MacBook Pro with notch |

---

## Open Questions

1. **Does `tauri-plugin-positioner` work correctly on macOS with the Dynamic Island / notch area?**
   - What we know: Plugin uses tray position + window size for placement
   - What's unclear: Whether it accounts for the notch cutout on MacBook Pro 2021+
   - Recommendation: Test on notch hardware; if off, add a manual Y offset

2. **Does the close-to-tray intercept need to be conditional on Windows only?**
   - What we know: On macOS, close behavior doesn't kill the process
   - What's unclear: Whether `prevent_close()` on macOS causes any side-effect (e.g., Cmd+Q not working)
   - Recommendation: Apply `prevent_close` universally (both platforms) — D-05 says both platforms hide to tray on close. Cmd+Q and tray Quit menu remain as the exit paths (D-06).

3. **Is there an existing codebase to scaffold from or is this truly greenfield?**
   - What we know: No `src-tauri/` or `src/` directories exist in the repo
   - What's unclear: Nothing — confirmed greenfield
   - Recommendation: Wave 0 first task is `pnpm create tauri-app speakly --template react-ts`

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `tauri::SystemTray` + `SystemTrayMenu` (v1 API) | `TrayIconBuilder` + `tauri::menu::Menu` (v2 API) | Tauri 2.0 (Oct 2024) | v1 code examples don't compile; use only v2 docs |
| `tauri-plugin-store` `store.set()` then `store.save()` (v1) | Auto-save on `store.set()` in v2, or `load(..., { autoSave: true })` | Tauri plugin-store v2 | No manual `save()` calls needed |
| `window.__TAURI__.invoke()` (v1 global) | `import { invoke } from '@tauri-apps/api/core'` (v2 ESM) | Tauri 2.0 | Must use named imports; global is gone |

**Deprecated / outdated:**
- `tauri::SystemTray`: Removed in v2. Replaced by `TrayIconBuilder`. All v1 tray tutorials are invalid.
- `#[tauri::command] fn` without `async`: Still works but async is preferred for non-blocking commands.

---

## Sources

### Primary (HIGH confidence)
- [Tauri System Tray v2](https://v2.tauri.app/learn/system-tray/) — tray setup, click events, TrayIconBuilder API
- [Tauri Store Plugin v2](https://v2.tauri.app/plugin/store/) — setup, TypeScript API (load, get, set), Rust StoreExt
- [Tauri Positioner Plugin v2](https://v2.tauri.app/plugin/positioner/) — tray-relative positioning, on_tray_event integration
- [Tauri Global Shortcut Plugin v2](https://v2.tauri.app/plugin/global-shortcut/) — register API, capabilities config
- [.planning/research/STACK.md](../.planning/research/STACK.md) — verified stack decisions (tray bug #13770, plugin choices)
- [.planning/research/PITFALLS.md](../.planning/research/PITFALLS.md) — Windows close behavior, hotkey silent failure
- [.planning/research/ARCHITECTURE.md](../.planning/research/ARCHITECTURE.md) — IPC patterns, AppState design

### Secondary (MEDIUM confidence)
- [GitHub discussion: Tray-only app close behavior (Tauri 2)](https://github.com/tauri-apps/tauri/discussions/11489) — `prevent_close()` pattern confirmed by community
- [tauritutorials.com: Building a system tray app](https://tauritutorials.com/blog/building-a-system-tray-app-with-tauri) — practical examples cross-referenced with official docs

### Tertiary (LOW confidence — flagged for validation)
- A1–A4 in Assumptions Log above

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all libraries verified via official Tauri v2 docs and prior STACK.md research
- Architecture patterns: HIGH — code examples derived from official docs + verified community patterns
- Pitfalls: HIGH — sourced from PITFALLS.md (already researched) + official Tauri GitHub issues
- Environment: HIGH for Node/pnpm; LOW for Rust (not installed, install path ASSUMED)

**Research date:** 2026-04-09
**Valid until:** 2026-05-09 (Tauri releases frequently; re-verify if tauri > 2.10.x releases before implementation)
