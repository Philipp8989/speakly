# Phase 2: Settings & Onboarding - Research

**Researched:** 2026-04-09
**Domain:** Tauri 2 multi-window, macOS permissions, React step wizard, API key validation
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** First launch shows a step-by-step onboarding wizard (3-4 screens, not a single page).
- **D-02:** Onboarding order: 1) Welcome + hotkey config, 2) API keys (OpenAI + Anthropic), 3) Microphone selection + permissions check, 4) Done/ready screen.
- **D-03:** Onboarding only shows on first launch (check ConfigStore `onboarding_complete` flag). User can re-run from settings.
- **D-04:** Each step has a "Weiter" (Next) and "Zurueck" (Back) button. Skip is allowed for optional steps (API keys can be entered later).
- **D-05:** Settings open in a separate Tauri window (not in the tray popup). Triggered by the gear icon in TrayPopup.
- **D-06:** Single scrollable page with sections: Hotkey, API Keys, Mikrofon, Sprache, Berechtigungen. No tabs.
- **D-07:** Settings window size ~500x600px, centered on screen, closeable (hides, doesn't quit app).
- **D-08:** User can remap the global hotkey from settings. Show current hotkey, click to record new one.
- **D-09:** Validate hotkey doesn't conflict. If conflict, show warning but allow anyway.
- **D-10:** Hotkey change takes effect immediately (unregister old, register new via plugin-global-shortcut).
- **D-11:** Password-masked input fields for OpenAI and Anthropic API keys.
- **D-12:** "Testen" button validates each key with a lightweight API call. Shows success/error inline.
- **D-13:** Keys stored in ConfigStore (settings.json). No keychain integration in v1.
- **D-14:** Dropdown listing available audio input devices. Default = system default.
- **D-15:** Microphone selection stored in ConfigStore. Applied in Phase 3.
- **D-16:** On macOS, check Microphone permission and Accessibility permission on app start and during onboarding.
- **D-17:** If permission denied, show status indicator (red/green) and "Einstellungen oeffnen" button that deep-links to System Preferences > Privacy & Security.
- **D-18:** Accessibility permission needed for enigo paste simulation (Phase 4). Check it early.
- **D-19:** On Windows, no special permissions needed — skip permission section or show "Alle Berechtigungen erteilt".

### Claude's Discretion
- Exact visual design of onboarding screens and settings page
- Animation/transitions between onboarding steps
- Microphone device enumeration method (cpal from Rust side via Tauri command)

### Deferred Ideas (OUT OF SCOPE)
None — discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| APP-02 | User kann Hotkey in den Einstellungen aendern | plugin-global-shortcut unregister/register pattern; hotkey capture via keydown listener |
| APP-04 | User kann API-Keys eingeben (OpenAI fuer Whisper, Anthropic fuer Claude) | ConfigStore already in place; OpenAI GET /v1/models + Anthropic GET /v1/models for key validation |
| APP-07 | Erster Start zeigt Onboarding (Hotkey, API-Keys, Mikrofon, Permissions) | WebviewWindow creation; onboarding_complete flag in ConfigStore; tauri-plugin-macos-permissions |
</phase_requirements>

---

## Summary

Phase 2 builds a settings/onboarding window as a second Tauri webview window. The primary mechanics are: (1) creating a new Tauri window via `WebviewWindow` from TypeScript, (2) implementing a 4-step wizard with local React state, (3) validating API keys via lightweight GET /v1/models requests from Rust, (4) checking macOS permissions via `tauri-plugin-macos-permissions`, and (5) enumerating microphone devices via a Rust command using `cpal`.

All core infrastructure (ConfigStore, global-shortcut plugin, Tauri 2 window system) is already in place from Phase 1. Phase 2 extends it with new ConfigStore keys, a new window label, new capabilities entries, and new Tauri commands for permissions and device enumeration.

**Primary recommendation:** Create the settings/onboarding window from TypeScript using `WebviewWindow` with label `"settings"`. Add `"core:webview:allow-create-webview-window"` to capabilities. Route between onboarding wizard and settings page using a `mode` prop or URL parameter (`?mode=onboarding` vs `?mode=settings`). Keep wizard state in a single React `useState` — no library needed for 4 steps.

---

## Standard Stack

### Core (all already installed in Phase 1)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `@tauri-apps/api` (WebviewWindow) | ^2 | Create settings window from frontend | Official Tauri JS API — only correct approach |
| `@tauri-apps/plugin-store` | ^2.4.2 | Persist all settings | Already in use; extend with new keys |
| `@tauri-apps/plugin-global-shortcut` | ^2.3.1 | Unregister old hotkey, register new | Already installed; unregister API documented |
| `@tauri-apps/plugin-opener` | ^2 | Open macOS System Preferences deep links | Already installed; use `open()` with URL scheme |
| React + TypeScript | 19.x / 5.x | Wizard step state, form inputs | Already in use |
| Tailwind CSS | ^4.x | Styling the new window | Already in use |

### New Dependencies (Phase 2 adds)

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `tauri-plugin-macos-permissions` (Rust) | latest | Check Microphone + Accessibility permission status | macOS only; `#[cfg(target_os = "macos")]` guard required |
| `tauri-plugin-macos-permissions-api` (JS) | latest | JS bindings for permission checks | Only consumed on macOS at runtime |
| `cpal` (Rust) | 0.16.0 | Enumerate audio input devices | Already in STACK.md; use `default_host().input_devices()` |

**Installation (new additions only):**
```bash
# Rust side
cargo add tauri-plugin-macos-permissions

# Frontend
pnpm add tauri-plugin-macos-permissions-api
```

`cpal` is already planned but may not be in Cargo.toml yet — add if not present:
```bash
cargo add cpal
```

### Version Verification

Verified versions at time of research:
- `tauri-plugin-macos-permissions` v2.3.0 [VERIFIED: GitHub ayangweb/tauri-plugin-macos-permissions, May 2025]
- `@tauri-apps/plugin-store` 2.4.2 [VERIFIED: package.json in repo]
- `@tauri-apps/plugin-global-shortcut` 2.3.1 [VERIFIED: package.json in repo]

---

## Architecture Patterns

### Recommended Project Structure (additions to existing src/)

```
src/
├── components/
│   ├── TrayPopup.tsx           # Phase 1 — wire onSettingsClick to openSettingsWindow()
│   ├── SettingsPage.tsx        # NEW — scrollable settings (D-06)
│   └── onboarding/
│       ├── OnboardingWizard.tsx  # NEW — step container + navigation
│       ├── StepWelcome.tsx       # NEW — step 1: welcome + hotkey
│       ├── StepApiKeys.tsx       # NEW — step 2: OpenAI + Anthropic keys
│       ├── StepMicrophone.tsx    # NEW — step 3: microphone + permissions
│       └── StepDone.tsx          # NEW — step 4: ready screen
├── hooks/
│   └── useSettings.ts          # NEW — read/write ConfigStore, share across pages
└── App.tsx                     # Phase 1 — add openSettingsWindow(), check onboarding flag
```

### Pattern 1: Creating the Settings Window from TypeScript

The settings window is opened from `App.tsx` when `handleSettingsClick` fires (already stubbed in Phase 1).

```typescript
// Source: https://v2.tauri.app/reference/javascript/api/namespacewebviewwindow/
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';

async function openSettingsWindow(mode: 'settings' | 'onboarding' = 'settings') {
  // Fenster nur einmal oeffnen — vorhandenes focussieren
  const existing = await WebviewWindow.getByLabel('settings');
  if (existing) {
    await existing.show();
    await existing.setFocus();
    return;
  }

  const win = new WebviewWindow('settings', {
    title: 'Speakly Einstellungen',
    url: `index.html?window=settings&mode=${mode}`,
    width: 500,
    height: 600,
    center: true,
    resizable: false,
    decorations: true,
    closable: true,
  });

  win.once('tauri://created', () => {
    console.log('Speakly: Settings-Fenster erstellt');
  });
  win.once('tauri://error', (e) => {
    console.error('Speakly: Settings-Fenster Fehler:', e);
  });
}
```

**Required capabilities entry** — add to `src-tauri/capabilities/default.json`:
```json
"core:webview:allow-create-webview-window",
"core:window:allow-show",
"core:window:allow-set-focus"
```

Also add `"settings"` to the `windows` array in the capability:
```json
"windows": ["main", "settings"]
```

**Window routing in App.tsx:** Read `?window=settings&mode=onboarding` from `window.location.search` to decide which component to render.

### Pattern 2: Onboarding Wizard — No Library Needed for 4 Steps

```typescript
// Source: React useState pattern — ASSUMED based on standard React practice
// Kein externes Wizard-Package noetig fuer 4 Schritte

const STEPS = ['welcome', 'api-keys', 'microphone', 'done'] as const;
type Step = typeof STEPS[number];

function OnboardingWizard() {
  const [currentStep, setCurrentStep] = useState<number>(0);

  const goNext = () => setCurrentStep(s => Math.min(s + 1, STEPS.length - 1));
  const goBack = () => setCurrentStep(s => Math.max(s - 1, 0));

  return (
    <div>
      {currentStep === 0 && <StepWelcome onNext={goNext} />}
      {currentStep === 1 && <StepApiKeys onNext={goNext} onBack={goBack} />}
      {currentStep === 2 && <StepMicrophone onNext={goNext} onBack={goBack} />}
      {currentStep === 3 && <StepDone />}
    </div>
  );
}
```

**Why no library:** All wizard libs (react-step-wizard, react-multistep) add bundle weight for functionality that 4 conditional renders + `useState` handles trivially.

### Pattern 3: ConfigStore Extensions

Add these keys in `initStore()` in App.tsx (and in the Rust `setup()` defaults):

```typescript
// Neue Standardwerte beim Erststart
if (!initialized) {
  await store.set('hotkey', DEFAULT_HOTKEY);
  await store.set('stt_mode', 'cloud');
  await store.set('openai_api_key', '');
  await store.set('anthropic_api_key', '');
  await store.set('microphone_id', 'default');
  await store.set('language', 'auto');
  await store.set('onboarding_complete', false);
  await store.set('initialized', true);
}
```

Reading settings in a hook:
```typescript
// Source: https://v2.tauri.app/plugin/store/
import { load } from '@tauri-apps/plugin-store';

export async function useSettingsStore() {
  const store = await load('settings.json', { autoSave: true });
  return store;
}
```

### Pattern 4: Hotkey Re-Registration (D-08, D-10)

```typescript
// Source: https://v2.tauri.app/plugin/global-shortcut/
import { register, unregister, isRegistered } from '@tauri-apps/plugin-global-shortcut';

async function remapHotkey(oldHotkey: string, newHotkey: string): Promise<boolean> {
  try {
    // Alten Hotkey freigeben
    if (await isRegistered(oldHotkey)) {
      await unregister(oldHotkey);
    }
    // Neuen Hotkey registrieren
    await register(newHotkey, (shortcut) => {
      // Phase 3 ersetzt diesen Handler
      console.log('Hotkey ausgeloest:', shortcut);
    });
    return true;
  } catch (err) {
    // Konflikt — Hotkey bereits von anderer App belegt
    console.warn('Hotkey-Konflikt:', err);
    return false;
  }
}
```

**Hotkey capture UI:** Use a `<input onKeyDown>` that captures modifier + key combination and formats it as Tauri shortcut string (e.g., `"CommandOrControl+Shift+R"`).

### Pattern 5: API Key Validation via Rust Commands

Both validations must run from Rust (not frontend fetch) to avoid CORS and CSP restrictions in Tauri.

```rust
// OpenAI: GET /v1/models — zero-cost, confirms key is valid
#[tauri::command]
async fn validate_openai_key(api_key: String) -> Result<bool, String> {
    let client = reqwest::Client::new();
    let resp = client
        .get("https://api.openai.com/v1/models")
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    Ok(resp.status().is_success())
}

// Anthropic: GET /v1/models — recommended over POST /v1/messages
// (POST fails with 404 if model name goes stale; GET /v1/models is always safe)
#[tauri::command]
async fn validate_anthropic_key(api_key: String) -> Result<bool, String> {
    let client = reqwest::Client::new();
    let resp = client
        .get("https://api.anthropic.com/v1/models")
        .header("x-api-key", &api_key)
        .header("anthropic-version", "2023-06-01")
        .send()
        .await
        .map_err(|e| e.to_string())?;
    Ok(resp.status().is_success())
}
```

**Key insight:** Use `GET /v1/models` for both providers. For Anthropic, avoid `POST /v1/messages` with a hardcoded model name — it fails with 404 if the model name becomes stale. [CITED: github.com/n8n-io/n8n/pull/27951]

### Pattern 6: macOS Permissions Check

```rust
// Rust side — nur auf macOS kompilieren
#[cfg(target_os = "macos")]
#[tauri::command]
async fn check_macos_permissions() -> serde_json::Value {
    use tauri_plugin_macos_permissions::PermissionsExt;
    // Plugin-API wird in setup() registriert
    serde_json::json!({
        "microphone": "granted",  // oder "denied" / "undetermined"
        "accessibility": "granted"
    })
}
```

TypeScript (via JS bindings):
```typescript
// Source: github.com/ayangweb/tauri-plugin-macos-permissions [VERIFIED v2.3.0]
import { checkMicrophonePermission, checkAccessibilityPermission } from 'tauri-plugin-macos-permissions-api';

const mic = await checkMicrophonePermission();   // "granted" | "denied" | "undetermined"
const ax  = await checkAccessibilityPermission(); // "granted" | "denied" | "undetermined"
```

**Deep-link to System Preferences:**
```typescript
// Source: Apple URL scheme documented at gist.github.com/rmcdongit/f66ff91e0dad78d4d6346a75ded4b751
import { open } from '@tauri-apps/plugin-opener';

// Mikrofon
await open('x-apple.systempreferences:com.apple.preference.security?Privacy_Microphone');
// Bedienungshilfen (Accessibility)
await open('x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility');
```

`tauri-plugin-opener` is already installed in Phase 1 — no new dependency needed.

### Pattern 7: Microphone Device Enumeration (Claude's Discretion)

```rust
// Tauri-Command um verfuegbare Mikrofone aufzulisten
// Source: docs.rs/cpal/latest/cpal/ [VERIFIED]
#[tauri::command]
fn list_audio_input_devices() -> Vec<String> {
    use cpal::traits::{DeviceTrait, HostTrait};
    let host = cpal::default_host();
    host.input_devices()
        .map(|devices| {
            devices
                .filter_map(|d| d.name().ok())
                .collect()
        })
        .unwrap_or_default()
}
```

Returns names as strings — frontend stores the selected name in ConfigStore as `microphone_id`. Phase 3 uses the name to find the device via `host.input_devices().find(|d| d.name().ok().as_deref() == Some(&stored_name))`.

### Anti-Patterns to Avoid

- **Opening System Preferences URL from Rust:** Use `plugin-opener` from TypeScript instead — it already has the permission. Calling `std::process::Command::new("open")` from Rust sidesteps the plugin and may fail in sandboxed builds.
- **Using `POST /v1/messages` for Anthropic key validation:** Hardcoded model names go stale and return 404. Use `GET /v1/models`.
- **Calling `register()` without first calling `unregister()`:** Double-registration throws an error. Always check `isRegistered()` or wrap in try/catch + unregister first.
- **Fetching API validation from frontend JS:** Tauri's CSP may block external fetch. Run all network calls through Rust Tauri commands.
- **Creating settings window unconditionally:** If gear icon is clicked twice, a second window spawns. Always check `WebviewWindow.getByLabel('settings')` first.
- **Storing API keys in ConfigStore without reading back:** ConfigStore is lazy — call `await store.save()` if `autoSave` is not set, or always use `{ autoSave: true }` in `load()`.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| macOS permission status | Custom `invoke('check_permissions')` Rust command from scratch | `tauri-plugin-macos-permissions` | Handles objc2 bindings, undetermined/granted/denied states correctly |
| Audio device list | Parse system audio info manually | `cpal::default_host().input_devices()` | Handles CoreAudio (macOS) + WASAPI (Windows) uniformly |
| API key network calls | Frontend `fetch()` | Rust `reqwest` Tauri commands | Avoids CSP/CORS issues in Tauri webview |
| Hotkey capture input | Raw keyboard state machine | `keydown` event + modifier key tracking in React | Sufficient; format to Tauri shortcut string manually |

---

## Common Pitfalls

### Pitfall 1: Settings Window Opened Multiple Times (D-07)

**What goes wrong:** User clicks the gear icon twice while the settings window is loading. Two windows appear.

**Why it happens:** `new WebviewWindow(...)` is async. The second click fires before `tauri://created` resolves, so `getByLabel()` still returns null.

**How to avoid:** Set a `isOpening` boolean flag in the click handler. Reset it on `tauri://created` or `tauri://error`. Only proceed if `!isOpening && getByLabel('settings') == null`.

**Warning signs:** Two identical windows appear on fast double-click.

### Pitfall 2: macOS Permission Check Fails in Dev (Unsigned)

**What goes wrong:** Accessibility and Microphone permission status reads as "undetermined" on a dev machine even after granting them. Behavior differs from production build.

**Why it happens:** Dev builds run unsigned. macOS ties permission caching to the bundle identifier. Tauri dev mode uses a different identifier path than the signed bundle.

**How to avoid:** Test permission flows on a `tauri build`-compiled binary, not `tauri dev`. Document this in the task notes.

**Warning signs:** Permissions appear granted in System Preferences but app reports "undetermined".

### Pitfall 3: Hotkey Registration Fails Silently After Remap (D-10)

**What goes wrong:** User enters a new hotkey. `unregister` succeeds. `register` returns success but hotkey doesn't fire.

**Why it happens:** Another app (Spotlight, Slack) has grabbed the combination. Plugin returns success even when OS-level registration fails silently on some macOS versions.

**How to avoid:** After `register()`, immediately test by firing a test event or at minimum show a warning: "Hotkey gesetzt — falls er nicht funktioniert, wähle eine andere Kombination." (D-09 allows this.)

### Pitfall 4: ConfigStore `onboarding_complete` Not Written Before Window Closes

**What goes wrong:** User completes onboarding but closes the window before the `store.set('onboarding_complete', true)` write resolves. On next launch, onboarding shows again.

**Why it happens:** Async ConfigStore writes; window close fires before the promise resolves.

**How to avoid:** Write `onboarding_complete = true` on the "Weiter" click from step 3 (Microphone), not on the final Done screen close event. The Done screen is informational only.

### Pitfall 5: API Key Stored But Not Readable in Same Session

**What goes wrong:** User enters API key, clicks "Testen" — validation passes. User closes settings, re-opens — key appears empty.

**Why it happens:** `load()` called with `autoSave: false` (default), and `store.save()` never explicitly called.

**How to avoid:** Always use `{ autoSave: true }` in every `load()` call — consistent with Phase 1 pattern already established in App.tsx.

---

## Code Examples

### Check Onboarding State on App Start

```typescript
// In App.tsx useEffect — prueft ob Onboarding bereits abgeschlossen
async function checkOnboarding() {
  const store = await load('settings.json', { autoSave: true });
  const done = await store.get<boolean>('onboarding_complete');
  if (!done) {
    await openSettingsWindow('onboarding');
  }
}
```

### Close Settings Window (Hide, Don't Quit)

The settings window must hide on close, not quit. Add to the Rust `setup()` handler — or handle from TypeScript using the `CloseRequested` event:

```typescript
// Im SettingsPage/OnboardingWizard Root-Component
import { getCurrentWindow } from '@tauri-apps/api/window';

useEffect(() => {
  const win = getCurrentWindow();
  const unlisten = win.onCloseRequested(async (event) => {
    event.preventDefault();
    await win.hide();
  });
  return () => { unlisten.then(f => f()); };
}, []);
```

### Capabilities File (updated default.json)

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Standardberechtigungen fuer Speakly Desktop",
  "windows": ["main", "settings"],
  "permissions": [
    "core:default",
    "store:default",
    "positioner:default",
    "global-shortcut:allow-register",
    "global-shortcut:allow-unregister",
    "global-shortcut:allow-is-registered",
    "core:webview:allow-create-webview-window",
    "core:window:allow-show",
    "core:window:allow-set-focus",
    "core:window:allow-hide",
    "opener:default"
  ]
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Use `POST /v1/messages` for Anthropic key validation | Use `GET /v1/models` | 2025 — n8n PR #27951 | Avoids 404 when hardcoded model name goes stale |
| Wizard library (react-step-wizard) | Plain `useState` + conditional render | Always valid for ≤6 steps | Removes bundle dependency for trivial case |
| Check Accessibility permission ad-hoc in Phase 4 | Check in Phase 2 onboarding | Best practice per PITFALLS.md | User is prepared before paste simulation fails |

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Wizard state can be managed with plain `useState` for 4 steps — no library needed | Architecture Patterns | Low — worst case add react-step-wizard later |
| A2 | `WebviewWindow.getByLabel()` is synchronous and can detect an already-open window before creation | Architecture Patterns | Medium — if async, double-window bug occurs; add flag guard |
| A3 | Rust `reqwest` calls are not blocked by Tauri CSP in a non-sandboxed app | Don't Hand-Roll | Medium — if CSP blocks, need to configure `app.security.csp` |
| A4 | `tauri-plugin-macos-permissions` v2.3.0 is compatible with Tauri 2.10.3 | Standard Stack | Low — plugin targets Tauri v2; minor version mismatch unlikely to break |

---

## Open Questions

1. **Does `cpal` need to be initialized per-command or as a shared host?**
   - What we know: `cpal::default_host()` is cheap to call per request.
   - What's unclear: Whether repeated calls cause resource contention on macOS CoreAudio.
   - Recommendation: Create host per command call in Phase 2; revisit in Phase 3 when recording starts.

2. **Should the settings window use a separate HTML entry point or route via URL params?**
   - What we know: `url: 'index.html?window=settings'` works in Tauri's single-page app model.
   - What's unclear: Whether Vite's dev server correctly handles the query param routing.
   - Recommendation: Use URL param (`?window=settings&mode=onboarding`) and read with `new URLSearchParams(window.location.search)` in App.tsx. Simpler than a second entry point.

3. **Does `tauri-plugin-macos-permissions` need to be registered in `lib.rs` `setup()`?**
   - What we know: Most Tauri plugins require `.plugin(tauri_plugin_X::init())` registration.
   - What's unclear: Exact registration API for this community plugin.
   - Recommendation: Check plugin README on first implementation. Standard Tauri plugin pattern likely applies.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `@tauri-apps/plugin-store` | ConfigStore writes | ✓ | 2.4.2 | — |
| `@tauri-apps/plugin-global-shortcut` | Hotkey remap | ✓ | 2.3.1 | — |
| `@tauri-apps/plugin-opener` | macOS SysPref deep link | ✓ | ^2 | — |
| `cpal` (Rust) | Microphone enumeration | Not yet in Cargo.toml | 0.16.0 (planned) | — |
| `reqwest` (Rust) | API key validation HTTP | Not yet in Cargo.toml | 0.12+ (planned) | — |
| `tauri-plugin-macos-permissions` | macOS permission checks | Not yet installed | 2.3.0 | Skip section on Windows; show "Alle erteilt" |

**Missing dependencies with no fallback:**
- `cpal` — must be added to `Cargo.toml` before microphone enumeration command can be written
- `reqwest` — must be added before API key validation commands can be written

**Missing dependencies with fallback:**
- `tauri-plugin-macos-permissions` — Windows path works without it (D-19); add with `#[cfg(target_os = "macos")]` guard

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Manual + Tauri dev run (no automated test framework detected) |
| Config file | none |
| Quick run command | `pnpm tauri dev` |
| Full suite command | `pnpm tauri dev` + manual walkthrough |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| APP-02 | Hotkey remap persists and fires | manual smoke | `pnpm tauri dev` — remap hotkey, restart, verify | ❌ Wave 0 |
| APP-04 | API key saved + validated | manual smoke | Enter key, click "Testen", expect success/error indicator | ❌ Wave 0 |
| APP-07 | First launch shows onboarding; subsequent launches skip it | manual smoke | Delete settings.json, restart; then restart again | ❌ Wave 0 |

### Sampling Rate
- **Per task:** Manual `pnpm tauri dev` run covering the affected component
- **Per wave merge:** Full walkthrough: first-launch onboarding → settings → hotkey remap → API key test → microphone select
- **Phase gate:** All 4 onboarding steps reachable; settings page sections visible; ConfigStore keys written; permissions show correct status on macOS

### Wave 0 Gaps
- [ ] No automated test framework present — all validation is manual. Acceptable for a Tauri desktop app in MVP phase.

---

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | n/a — local desktop app, no user accounts |
| V3 Session Management | no | n/a |
| V4 Access Control | no | n/a |
| V5 Input Validation | yes | Validate hotkey format before `register()`; strip whitespace from API keys before storage |
| V6 Cryptography | partial | API keys stored in plaintext JSON (D-13 explicitly accepts this for v1); document as known limitation |

### Known Threat Patterns for This Stack

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| API key exfiltration via log output | Information Disclosure | Never log API key values; only log key presence (`key.is_empty()`) |
| API key exposure in process list | Information Disclosure | Keys passed as function args in Rust, not as CLI args — safe |
| Hotkey string injection | Tampering | Validate hotkey matches known modifier+key pattern before passing to `register()` |
| Malicious URL in Settings Preferences link | Tampering | Deep-link URLs are hardcoded constants, not user-provided |

---

## Sources

### Primary (HIGH confidence)
- [Tauri WebviewWindow API](https://v2.tauri.app/reference/javascript/api/namespacewebviewwindow/) — window creation, options, events
- [Tauri Store Plugin](https://v2.tauri.app/plugin/store/) — get/set/save pattern
- [Tauri Global Shortcut Plugin](https://v2.tauri.app/plugin/global-shortcut/) — register/unregister/isRegistered
- [cpal docs.rs](https://docs.rs/cpal/latest/cpal/) — `default_host().input_devices()`
- [tauri-plugin-macos-permissions GitHub](https://github.com/ayangweb/tauri-plugin-macos-permissions) — v2.3.0, checkMicrophonePermission / checkAccessibilityPermission
- [OpenAI List Models endpoint](https://platform.openai.com/docs/api-reference/models/list) — GET /v1/models for key validation
- [Apple System Preferences URL Schemes](https://gist.github.com/rmcdongit/f66ff91e0dad78d4d6346a75ded4b751) — x-apple.systempreferences deep links

### Secondary (MEDIUM confidence)
- [Anthropic key validation — use GET /v1/models](https://github.com/n8n-io/n8n/pull/27951) — confirmed via n8n PR, verified rationale
- [Tauri capabilities for WebviewWindow creation](https://github.com/tauri-apps/tauri/discussions/9487) — `core:webview:allow-create-webview-window` permission requirement

### Tertiary (LOW confidence — flag for validation)
- `WebviewWindow.getByLabel()` synchronous behavior assumption [A2]

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all libraries already installed or have verified versions
- Architecture: HIGH — WebviewWindow pattern from official docs; wizard pattern is standard React
- API validation: HIGH — GET /v1/models is documented and zero-cost for both providers
- macOS permissions: MEDIUM — tauri-plugin-macos-permissions is community plugin (verified v2.3.0, but integration details need confirmation in lib.rs setup)
- Pitfalls: HIGH — sourced from existing PITFALLS.md (verified) + phase-specific analysis

**Research date:** 2026-04-09
**Valid until:** 2026-05-09 (stable Tauri ecosystem; 30-day window)
