# Domain Pitfalls: Cross-Platform Voice-to-Text Desktop App

**Domain:** Voice-to-Text Desktop App (Tauri + Whisper + Clipboard paste)
**Researched:** 2026-04-09
**Confidence:** MEDIUM-HIGH (most verified via GitHub issues and official docs)

---

## Critical Pitfalls

Mistakes that cause rewrites, broken user trust, or complete feature failure.

---

### Pitfall 1: Clipboard Clobber — Silent Data Loss

**What goes wrong:** The app writes transcribed text to the clipboard, then triggers Cmd+V / Ctrl+V to paste. If the user had something copied before starting the recording, their clipboard content is permanently destroyed. No undo. They don't see it happen.

**Why it happens:** Clipboard is a global, single-slot buffer shared by all apps. The dictation flow reads: save → overwrite clipboard → simulate paste → restore. The restore step is timing-sensitive and often skipped in v1 implementations.

**Consequences:** User loses a copied URL, code snippet, or password. Immediate trust failure. Becomes a blocker for adoption.

**Prevention:**
1. Always save clipboard content before writing.
2. Restore clipboard after a short delay (50-100ms after paste event) — the delay is needed because some apps (Electron, web apps) read clipboard asynchronously.
3. Use a flag to detect if the restoration failed.
4. Show a visual indicator BEFORE paste so user knows the hotkey does something.

**Warning signs:** Any implementation that does `set_clipboard(text) → simulate_paste()` without a save/restore wrapper is at risk.

**Phase to address:** Phase 1 (core recording + paste loop). Must be correct from day one.

---

### Pitfall 2: macOS Microphone Permission Not Prompted After First Denial

**What goes wrong:** On macOS, if the user denies microphone access once — or if the app requests it incorrectly — the system never prompts again. The app silently gets empty audio. Transcription returns garbage or hallucinated text. User thinks STT is broken.

**Why it happens:** macOS caches permission decisions. A missing `NSMicrophoneUsageDescription` in Info.plist causes the prompt to never appear. A wrong entitlements configuration with app sandboxing can prevent access entirely.

**Consequences:** App appears broken on first run. Users who click "Don't Allow" on the first prompt can never use the app without going to System Preferences manually.

**Prevention:**
1. Add `NSMicrophoneUsageDescription` to `tauri.conf.json` → `bundle → macOS → infoPlist`.
2. On recording start, always check permission status first. If denied, show a dialog that deep-links to `x-apple.systempreferences:com.apple.preference.security?Privacy_Microphone`.
3. Test on a clean macOS account with no prior permission grants.
4. Do NOT rely on dev-mode behavior — permissions behave differently in signed builds.

**Warning signs:** Audio buffer is all zeros or recording duration is 0ms. No system dialog appears on first launch.

**Phase to address:** Phase 1. Test permission flow before anything else works.

---

### Pitfall 3: whisper.cpp Has 100x Performance Regression on Windows vs macOS

**What goes wrong:** whisper.cpp runs in 0.4s on Apple Silicon (M2) and 39.7s on an equivalent Windows laptop for the same audio. The offline fallback becomes unusable on Windows for any model above `tiny`.

**Why it happens:** Apple Silicon has Metal GPU acceleration and a highly optimized BLAS implementation. Windows lacks this by default — without AVX2 + OpenBLAS or CUDA, whisper.cpp runs single-threaded on CPU with no SIMD acceleration.

**Consequences:** Offline mode is unusable on Windows. "Offline fallback" feature doesn't work for half the team.

**Prevention:**
1. Ship pre-compiled whisper.cpp binaries with OpenBLAS enabled for Windows x64.
2. Default to `tiny` or `base` model for offline mode — not `medium` or `large`.
3. Warn users on Windows that offline mode is slower and recommend Cloud API as primary.
4. Consider: offline mode is a stretch goal for v1 on Windows — verify before committing.
5. Never cross-compile whisper.cpp — build per-platform in CI.

**Warning signs:** `ggml_backend_cpu_init` in whisper.cpp logs (no GPU acceleration). Transcription taking >5s for a 10s clip.

**Phase to address:** Phase with offline/local model implementation. Requires platform-specific build pipeline.

---

### Pitfall 4: Global Hotkey Registration Silently Fails

**What goes wrong:** `tauri-plugin-global-shortcut` returns success but the hotkey doesn't work because another app (e.g., Spotlight = Cmd+Space, Slack, VS Code) has already registered the same combination. On macOS, some system shortcuts cannot be overridden at all. The user sees no error.

**Why it happens:** The OS allows only one app to own a global hotkey combination. No error is thrown in many cases — registration silently no-ops or gets stolen.

**Consequences:** The core feature (hotkey → record) doesn't work. User doesn't know why. Looks like a bug.

**Prevention:**
1. Always handle registration failure explicitly — check the return value / catch errors.
2. Show a visible notification if registration fails: "Hotkey Cmd+Shift+Space is taken. Please choose a different one."
3. Provide a settings page for hotkey remapping from day one, not as an afterthought.
4. Avoid common conflicts: `Cmd+Space` (Spotlight), `Cmd+Shift+Space` (various), `Ctrl+Shift+S` (many apps).
5. Default to an unusual combination like `Ctrl+Shift+Alt+R` to minimize conflicts.

**Warning signs:** Hotkey works in dev but not in a signed/installed build. Works on dev machine but not on teammate's machine.

**Phase to address:** Phase 1. Hotkey remapping UI must exist before sharing with any user.

---

### Pitfall 5: Whisper Hallucinations on Silence / Short Audio

**What goes wrong:** If the user presses the hotkey but says nothing (or the recording is very short, e.g. <0.5s), Whisper returns fabricated text — common outputs include "Thank you.", "Subtitles by...", "I'm sorry.", or repeated phrases. This text gets pasted into whatever was active.

**Why it happens:** Whisper was trained on subtitle data. Short or silent audio activates "fallback completion" behavior where the model generates training-data-like text rather than returning empty.

**Consequences:** Garbage text pasted into user's document, email, or code. Extremely confusing UX. Hard to explain to non-technical users.

**Prevention:**
1. Implement Voice Activity Detection (VAD) before sending audio to Whisper. Only send if speech is detected.
2. Set a minimum recording duration (e.g., 0.5s of actual audio above noise floor).
3. For local whisper.cpp: use `--no-timestamps` and check if output contains known hallucination strings.
4. Strip leading/trailing silence from audio before upload to Whisper API.
5. If transcript is less than 3 characters or matches known hallucination patterns, discard silently.

**Warning signs:** Identical short texts appearing in random places. "Thank you." or "Subtitles by" appearing in output.

**Phase to address:** Phase 1 (recording pipeline). VAD is not optional — it's a correctness requirement.

---

## Moderate Pitfalls

---

### Pitfall 6: Text Insertion Fails in Certain App Types

**What goes wrong:** The clipboard + Cmd+V paste approach fails silently in several app categories:
- **Password managers** (1Password, Bitwarden) — block paste from simulated input
- **Electron apps with custom paste handlers** — may discard or double-process
- **Terminal apps** (iTerm2) — require Cmd+Shift+V or a different paste key
- **Browser address bars** — may have paste suppression
- **Full-screen games** — have exclusive input capture

**Why it happens:** Paste simulation via `enigo` or `rdev` sends OS-level keyboard events. Apps with custom input sanitization or focus quirks may reject or mis-handle them.

**Prevention:**
1. Test in the top 10 target apps for the team: Slack, VS Code, Gmail (Chrome), Notion, Terminal.
2. Document known limitations clearly rather than trying to hack around all edge cases.
3. Consider platform accessibility APIs as a fallback: `AXUIElement` on macOS for known apps.
4. Show status indicator (tray icon change) during paste so user knows something happened.

**Warning signs:** Paste works in TextEdit/Notepad but not in Slack or VS Code.

**Phase to address:** Phase 1 testing. Phase 2 if edge-case handling is needed.

---

### Pitfall 7: macOS Accessibility Permission Required for Paste Simulation

**What goes wrong:** On macOS, simulating keyboard events (Cmd+V) requires Accessibility permission (`com.apple.security.automation.apple-events` or direct AX permission). Without it, the paste simulation silently does nothing in sandboxed or protected apps.

**Why it happens:** macOS restricts programs from sending keystrokes to other apps without explicit user trust. This is separate from microphone permission.

**Consequences:** Paste works in dev (usually unsigned, not sandboxed), fails in production for users who didn't grant Accessibility access.

**Prevention:**
1. On first launch, check Accessibility permission status via `tauri-plugin-macos-permissions`.
2. If not granted, show an onboarding screen explaining why it's needed and deep-linking to settings.
3. Do not enable the hotkey until both Microphone and Accessibility permissions are confirmed.
4. Test on a freshly provisioned macOS machine with default security settings.

**Warning signs:** App works on dev machine but paste does nothing on a fresh install.

**Phase to address:** Phase 1 (onboarding / permission setup flow).

---

### Pitfall 8: Whisper API File Size Limit (25MB) for Long Recordings

**What goes wrong:** Uncompressed WAV at 16kHz mono maxes out the 25MB limit at approximately 90 seconds. A stereo recording at 44.1kHz hits it in under 10 seconds. Users who record a long dictation get a silent failure or API error.

**Why it happens:** OpenAI Whisper API hard-caps at 25MB. WAV is uncompressed. Stereo doubles size with no quality benefit for speech.

**Prevention:**
1. Always record as mono, not stereo.
2. Always record at 16kHz, not 44.1kHz — Whisper internally resamples to 16kHz anyway.
3. Use 16-bit PCM, not 32-bit float.
4. Enforce a maximum recording duration in the UI (e.g., 5 minute warning at 4 minutes).
5. Optionally encode to mp3/opus before upload to stay well within limits and reduce latency.

**Warning signs:** API returns 400 error with "file too large". Audio file size growing faster than expected.

**Phase to address:** Phase 1 (audio capture configuration). Get this right in the audio pipeline from day one.

---

### Pitfall 9: Tauri macOS Notarization Blocks Shipping

**What goes wrong:** An un-notarized macOS app shows "Apple could not verify that this app is free from malware" and is blocked from running by Gatekeeper on every user's machine. Requires right-click → Open, and still shows a warning. Feels unprofessional and breaks team trust.

**Why it happens:** Apple requires notarization for all apps distributed outside the Mac App Store. This requires an Apple Developer account ($99/yr) and a working CI notarization step.

**Prevention:**
1. Budget for Apple Developer Program enrollment before targeting macOS production.
2. Set up notarization in CI (GitHub Actions) early — don't leave it to last.
3. Use `tauri build --target universal-apple-darwin` for a universal binary (Intel + Apple Silicon) to avoid separate builds.
4. Test notarized build on a separate machine before any team rollout.

**Warning signs:** Build works on your machine (because your machine trusts the dev cert) but users report "can't open" errors.

**Phase to address:** Distribution phase. Plan signing/notarization setup before any team rollout.

---

### Pitfall 10: Windows SmartScreen Warning Blocks New Installs

**What goes wrong:** On Windows, unsigned or new-publisher installers trigger SmartScreen: "Windows protected your PC. This app might put your PC at risk." Most non-technical users click "Don't run." EV certificates cost $400+/yr.

**Why it happens:** Microsoft SmartScreen builds reputation per certificate. A new certificate starts with zero reputation.

**Prevention:**
1. Sign the Windows installer even without EV — self-signed via Azure Key Vault reduces friction.
2. Submit the binary to Microsoft's file submission portal to accelerate reputation building.
3. Include instructions in the README for bypassing SmartScreen on first install.
4. If budget allows, an EV certificate from DigiCert/Sectigo eliminates the warning immediately.

**Warning signs:** All team installs on Windows require "Run anyway" confirmation.

**Phase to address:** Distribution phase, but plan signing infrastructure before first Windows build.

---

## Minor Pitfalls

---

### Pitfall 11: rdev Keyboard Events Break When Tauri Window Is Focused

**What goes wrong:** If using `rdev` for global hotkey listening alongside Tauri, keyboard events stop firing when the Tauri window itself gets focus. Mouse events continue working. This is a known Tauri+rdev interaction bug (issue #14770).

**Prevention:** Use `tauri-plugin-global-shortcut` (official plugin) instead of `rdev` for hotkey detection. If rdev is needed, run it in a separate thread and test focus edge cases.

**Phase to address:** Phase 1 (hotkey implementation). Use the right library upfront.

---

### Pitfall 12: Windows Background Process Killed When Window Closes

**What goes wrong:** On Windows, closing the main window kills the process entirely. On macOS, the app keeps running. This means the global hotkey stops working when the user closes the window on Windows — which is the primary usage mode (tray-only app).

**Prevention:** Intercept the close event on Windows and minimize to tray instead:
```rust
window.on_window_event(|event| {
  if let WindowEvent::CloseRequested { api, .. } = event {
    api.prevent_close();
    window.hide().unwrap();
  }
});
```
**Phase to address:** Phase 1 (app lifecycle). Test on Windows from day one.

---

### Pitfall 13: Audio Format Mismatch Causes Silent Transcription Degradation

**What goes wrong:** Recording in WebM/Opus (common browser default) and sending to Whisper API works but produces worse results than WAV 16kHz mono. The API re-encodes internally, introducing artifacts. Users see higher error rates but no obvious cause.

**Prevention:** Always record and send audio in WAV format with 16kHz sample rate, mono channel, 16-bit PCM. Do not use the browser's MediaRecorder default encoding. Explicitly configure the audio capture format in Rust.

**Phase to address:** Phase 1 (audio capture pipeline).

---

### Pitfall 14: Tauri Cannot Cross-Compile — CI Required Per Platform

**What goes wrong:** Developers on Mac assume they can build the Windows binary locally via `cargo build --target x86_64-pc-windows-msvc`. This fails — Tauri depends on platform-native libraries (WiX, NSIS, WebView2, AppKit) that don't cross-compile.

**Prevention:**
1. Set up GitHub Actions with matrix builds (macOS runner for .dmg, Windows runner for .msi/.exe) from the start.
2. Never promise a "build both platforms locally" workflow.
3. Use `tauri-action` in CI for consistent signed/notarized artifacts.

**Phase to address:** Before any distribution milestone.

---

## Phase-Specific Warning Map

| Phase Topic | Likely Pitfall | Mitigation |
|-------------|---------------|------------|
| Audio capture setup | Microphone permission silent failure on macOS | Check + request permission before first record |
| Audio capture setup | Wrong audio format (stereo, 44.1kHz, WebM) | Configure explicitly: 16kHz mono WAV |
| Hotkey implementation | Silent registration failure | Always handle errors, provide fallback + settings |
| Hotkey implementation | rdev + Tauri window focus bug | Use tauri-plugin-global-shortcut only |
| Paste simulation | Clipboard clobber / data loss | Save + restore clipboard with timing delay |
| Paste simulation | macOS Accessibility permission missing | Onboarding flow, check before enabling feature |
| STT integration | Whisper hallucinations on silence | VAD pre-check, minimum audio duration |
| STT integration | 25MB file size limit exceeded | Mono 16kHz, enforce max duration |
| Offline model | whisper.cpp 100x slowdown on Windows | Only tiny/base model, warn user, make optional |
| App lifecycle | Window close kills app on Windows | Intercept close → hide to tray |
| Distribution (macOS) | Gatekeeper blocks unsigned app | Notarization pipeline before team rollout |
| Distribution (Windows) | SmartScreen blocks new installs | Sign + submit for reputation building |
| CI/Build pipeline | Cross-compilation impossible | Platform-native CI runners from day one |

---

## Sources

- Tauri macOS microphone permission bug: https://github.com/tauri-apps/tauri/issues/11951
- Tauri macOS microphone access from Rust: https://github.com/tauri-apps/tauri/issues/9928
- tauri-plugin-macos-permissions: https://github.com/ayangweb/tauri-plugin-macos-permissions
- Global shortcut plugin docs: https://v2.tauri.app/plugin/global-shortcut/
- Global hotkey registration failure discussion: https://github.com/tauri-apps/tauri/discussions/10017
- rdev + Tauri window focus bug: https://github.com/tauri-apps/tauri/issues/14770
- whisper.cpp 100x Windows slowdown: https://github.com/ggml-org/whisper.cpp/issues/936
- Whisper hallucinations on silence: https://github.com/ggml-org/whisper.cpp/issues/1724
- Whisper hallucination discussion: https://github.com/openai/whisper/discussions/1606
- Whisper API audio format optimization: https://dev.to/mxro/optimise-openai-whisper-api-audio-format-sampling-rate-and-quality-29fj
- Tauri Windows code signing: https://v2.tauri.app/distribute/sign/windows/
- Tauri macOS code signing: https://v2.tauri.app/distribute/sign/macos/
- Shipping Tauri v2 app macOS notarization: https://dev.to/massi_24/shipping-a-production-macos-app-with-tauri-20-code-signing-notarization-and-homebrewpublished-o10
- Tauri system tray focus bug: https://github.com/tauri-apps/tauri/issues/7884
- Tauri sidecar binaries: https://v2.tauri.app/develop/sidecar/
- Cross-platform compilation Tauri: https://v1.tauri.app/v1/guides/building/cross-platform/
