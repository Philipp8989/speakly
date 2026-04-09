# Phase 2: Settings & Onboarding - Context

**Gathered:** 2026-04-09
**Status:** Ready for planning

<domain>
## Phase Boundary

Deliver the settings UI and first-run onboarding flow. User can configure hotkey, enter API keys, select microphone, and grant required permissions. After onboarding, the app is ready for recording (Phase 3).

</domain>

<decisions>
## Implementation Decisions

### Onboarding Flow
- **D-01:** First launch shows a step-by-step onboarding wizard (3-4 screens, not a single page).
- **D-02:** Onboarding order: 1) Welcome + hotkey config, 2) API keys (OpenAI + Anthropic), 3) Microphone selection + permissions check, 4) Done/ready screen.
- **D-03:** Onboarding only shows on first launch (check ConfigStore `onboarding_complete` flag). User can re-run from settings.
- **D-04:** Each step has a "Weiter" (Next) and "Zurueck" (Back) button. Skip is allowed for optional steps (API keys can be entered later).

### Settings Window
- **D-05:** Settings open in a separate Tauri window (not in the tray popup). Triggered by the gear icon in TrayPopup (wired as stub in Phase 1 D-04).
- **D-06:** Single scrollable page with sections: Hotkey, API Keys, Mikrofon, Sprache, Berechtigungen. No tabs needed — few enough settings.
- **D-07:** Settings window size ~500x600px, centered on screen, closeable (hides, doesn't quit app).

### Hotkey Configuration
- **D-08:** User can remap the global hotkey from settings. Show current hotkey, click to record new one.
- **D-09:** Validate hotkey doesn't conflict. If conflict, show warning but allow anyway (user decides).
- **D-10:** Hotkey change takes effect immediately (unregister old, register new via plugin-global-shortcut).

### API Key Input
- **D-11:** Password-masked input fields for OpenAI API key and Anthropic API key.
- **D-12:** "Testen" button validates each key with a lightweight API call (e.g., list models for OpenAI, messages API for Anthropic). Shows success/error inline.
- **D-13:** Keys stored in ConfigStore (settings.json). Acceptable for local desktop app — no keychain integration in v1.

### Microphone Selection
- **D-14:** Dropdown listing available audio input devices. Default = system default.
- **D-15:** Microphone selection stored in ConfigStore. Applied in Phase 3 when cpal starts recording.

### Permission Handling (macOS)
- **D-16:** On macOS, check Microphone permission and Accessibility permission on app start and during onboarding.
- **D-17:** If permission denied, show status indicator (red/green) and "Einstellungen oeffnen" button that deep-links to System Preferences > Privacy & Security.
- **D-18:** Accessibility permission is needed for enigo paste simulation (Phase 4). Check it early so user is ready.
- **D-19:** On Windows, no special permissions needed — skip permission section or show "Alle Berechtigungen erteilt" (all granted).

### Claude's Discretion
- Exact visual design of onboarding screens and settings page
- Animation/transitions between onboarding steps
- Microphone device enumeration method (cpal from Rust side via Tauri command)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Existing Code
- `src/App.tsx` — Current app root with ConfigStore init and hotkey registration stub
- `src/components/TrayPopup.tsx` — Has `onSettingsClick` prop wired as stub
- `src-tauri/src/lib.rs` — Plugin setup, ConfigStore defaults, close-to-tray intercept
- `src-tauri/tauri.conf.json` — Window configuration, capabilities

### Research
- `.planning/research/STACK.md` — Plugin versions, ConfigStore API patterns
- `.planning/research/PITFALLS.md` — macOS permission gotchas, Accessibility requirement

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `ConfigStore` (plugin-store) already initialized in both Rust and TypeScript — extend with new keys
- `plugin-global-shortcut` already registered — needs unregister/re-register for hotkey change
- `TrayPopup.onSettingsClick` stub ready to wire to settings window creation

### Established Patterns
- German code comments (CLAUDE.md convention)
- Tauri IPC: Rust commands for system operations, TypeScript for UI
- ConfigStore keys: `hotkey`, `stt_mode`, `initialized` already exist

### Integration Points
- Settings gear in TrayPopup → opens new Tauri window with settings UI
- Onboarding wizard → same window, different route/component
- ConfigStore → add keys: `openai_api_key`, `anthropic_api_key`, `microphone_id`, `language`, `onboarding_complete`
- Permission check → Tauri command from Rust (check macOS permissions)

</code_context>

<specifics>
## Specific Ideas

- Onboarding should feel lightweight and fast — not a 10-minute setup wizard
- Permission deep-links to System Preferences are essential for macOS UX
- API key validation gives immediate feedback — user knows if key works before leaving settings

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 02-settings-onboarding*
*Context gathered: 2026-04-09*
