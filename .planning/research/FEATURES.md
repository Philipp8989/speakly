# Feature Landscape: Voice-to-Text Desktop App

**Domain:** Voice dictation desktop application (cross-platform)
**Researched:** 2026-04-09
**Competitors analyzed:** Voicy, VoiceInk, Wispr Flow, Spokenly, Voibe, macOS native dictation

---

## Table Stakes

Features users expect. Missing = product feels incomplete or unusable.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Global hotkey start/stop recording | Core interaction — everything else is secondary | Low | Must be user-configurable. Hold-to-record vs toggle are both valid, offer both. |
| Audio level / waveform feedback during recording | Users need confirmation the mic is live | Low | Spinner is not enough — waveform or animated bars show live input. PulseScribe specifically cited this. |
| Auto-paste into active text field | The entire value proposition — no manual paste step | Medium | Clipboard+Cmd+V fallback is acceptable; direct Accessibility API injection is preferred. Cascade: Accessibility API → Clipboard paste. |
| Automatic punctuation | Raw transcript without periods is unusable for real work | Low-Med | Whisper API handles this natively. Local whisper.cpp requires prompt engineering. |
| 50+ language support | International teams. Anyone with a non-English keyboard. | Low | Whisper natively supports 99 languages. Auto-detection is standard. |
| System tray / menu bar icon | Users need to know the app is running and its state | Low | Icon must change state: idle / recording / processing. Text-only status is insufficient. |
| Configurable hotkey | Default hotkey conflicts with other apps (Slack, IDEs) | Low | All competitors offer this. |
| Microphone selection | Teams use external mics, headsets, USB audio | Low | Dropdown in settings. Default = system default. |
| Privacy-first positioning | Privacy concerns are a top user objection to cloud STT | Low | Clear statement: no training on user audio. Voicy and EdgeWhisper lead with this. |
| API key setup for cloud STT | BYOK (Bring Your Own Key) is the de-facto model for open/free tools | Low | First-run setup must be frictionless. Show where to get the key. |
| Cross-platform (Mac + Windows) | Core requirement per PROJECT.md | High | Tauri already selected. Main risk is text insertion on Windows (no Accessibility API equivalent). |

---

## Differentiators

Features that set the product apart. Not table stakes, but add meaningful value.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| AI post-processing commands | "Make this formal", "translate to German", "shorter" — turns dictation into a writing assistant | Medium | Claude API already planned. Needs a clean UX — command palette or predefined templates. Most competitors have this but UX is clunky. |
| App-aware text insertion (cascade) | Terminal, VS Code, Electron apps each need different insertion methods. Cascade reduces support tickets. | Med-High | TypeVox and Steno do this. Clipboard → Accessibility API → keystroke simulation. |
| Offline fallback via whisper.cpp | Works without internet, appeals to privacy-conscious users and travelers | High | whisper.cpp is complex to bundle in Tauri. Model download on first run (need UX for this). |
| Hold-to-record (push-to-talk) mode | Faster for short dictations — no toggle state confusion | Low | Complementary to toggle mode. Users with tremors or hands-free workflows prefer toggle. |
| Status overlay / floating indicator | Small floating window showing "Recording..." or "Processing..." at cursor | Low-Med | Non-intrusive. PulseScribe does animated overlay. Users know where to look. |
| Custom vocabulary / personal dictionary | Names, acronyms, technical terms that STT misrecognizes | Medium | Whisper API supports a `prompt` parameter for vocabulary priming — lower complexity than true dictionary. |
| AI command templates / presets | Predefined commands users can pick from rather than freetyping | Low | Email, Slack, formal, casual — Voicy and WisprFlow offer this. Easy win. |
| Silence detection / auto-stop | Stops recording after N seconds of silence — no need to hit hotkey twice | Low | Standard in mature apps. Configurable timeout. |

---

## Anti-Features

Features to deliberately NOT build — they add complexity without proportional value for this use case.

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| Real-time streaming transcription | Significant latency/UX complexity. Users dictate a sentence, then get the text — batch is fine. PROJECT.md already excludes this. | Batch after recording stop. Acceptable 0.5–2s delay. |
| Transcript history / search | Adds persistent storage, sync, privacy surface, UI complexity. Not the core job-to-be-done. | Let the target app (Gmail, Notion, Docs) own history. |
| Team management / shared settings | Requires backend, auth, billing. Out of scope per PROJECT.md. | Users bring own API keys; config is local. |
| Voice commands to control the OS | Expands scope dramatically. Different product category (Dragon NaturallySpeaking territory). | Stick to "dictate text into focused field." |
| Built-in note-taking window | Creates competing UI with the apps users already have open | Text goes into whatever app is focused — no owned canvas. |
| Auto-punctuation editing UI | Inline correction of transcribed text before insertion adds a modal step that breaks flow | Trust the STT quality; users can edit in the target app. |
| Custom wake word ("Hey Speakly") | Always-on microphone, major privacy risk, complex implementation | Global hotkey is the correct activation model for desktop. |
| Subscription / licensing backend | No backend needed. User brings API keys. Project is internal/open-source. | BYOK. Costs fall on user's Whisper/Claude usage. |

---

## Feature Dependencies

```
Global Hotkey
  → Audio Recording (requires mic permission)
      → Visual Recording Feedback (waveform/overlay)
          → STT Transcription (Cloud Whisper API OR local whisper.cpp)
              → Auto-punctuation (handled by Whisper API; prompt-engineering for local)
              → AI Post-Processing (requires Claude API key, optional)
              → Text Insertion
                  → Accessibility API method (requires permission grant, Mac)
                  → Clipboard + Paste fallback (cross-platform, always works)

System Tray Icon
  → State management (idle / recording / processing / error)
  → Settings access (hotkey, mic, API keys, language)

API Key Setup (onboarding)
  → Whisper API key (required for cloud mode)
  → Claude API key (required for AI commands, optional)
  → Offline model download (required for local mode, optional)

Offline Fallback
  → whisper.cpp binary bundled with app
  → Model download on first use (300MB–1.5GB depending on model size)
```

---

## MVP Recommendation

**Prioritize (must ship):**
1. Global hotkey start/stop recording (toggle + hold-to-record)
2. Audio level visual feedback (waveform or animated indicator)
3. Whisper API transcription with auto-punctuation
4. Auto-paste via clipboard (Cmd+V simulation) — Accessibility API as enhancement
5. System tray icon with 3 states (idle / recording / processing)
6. First-run onboarding: hotkey + API key setup
7. Settings: hotkey, microphone, language, API key

**Ship in v1.1:**
- AI post-processing commands (Claude API)
- Silence auto-stop
- Offline fallback via whisper.cpp (complex model bundling)
- App-aware insertion cascade

**Defer:**
- Custom vocabulary: Medium complexity, low priority for initial team use case
- AI command templates/presets: Nice polish, not blocking adoption
- Status overlay at cursor: Useful but tray icon covers minimum need

---

## Confidence Notes

| Area | Confidence | Basis |
|------|------------|-------|
| Table stakes features | HIGH | Consistent across all competitors analyzed (Voicy, VoiceInk, Wispr Flow, Spokenly, Voibe) |
| Text insertion methods | HIGH | Multiple open-source implementations documented (Steno, TypeVox, EdgeWhisper) |
| Anti-features | HIGH | Derived from PROJECT.md out-of-scope list + competitor scope analysis |
| AI command UX patterns | MEDIUM | Feature exists in multiple competitors but UX best practices less documented |
| whisper.cpp offline complexity | MEDIUM | Known implementation challenge, Tauri bundling specifics need phase-specific research |

---

## Sources

- [Voicy — AI Speech to Text App](https://usevoicy.com/)
- [VoiceInk — GitHub (open source)](https://github.com/Beingpax/VoiceInk)
- [VoiceInk Review 2026](https://www.getvoibe.com/resources/voiceink-review/)
- [Wispr Flow vs VoiceInk comparison](https://wisprflow.ai/post/wispr-flow-vs-voiceink-2025)
- [Spokenly vs Voibe comparison](https://spokenly.app/comparison/voibe)
- [Best AI Dictation Apps 2025 — Local vs Cloud](https://www.implicator.ai/the-2025-buyers-guide-to-ai-dictation-apps-windows-macos-ios-android-linux/)
- [TypeVox — Dictation on Mac guide (text insertion methods)](https://typevox.app/blog/dictation-on-mac/)
- [PulseScribe — UX philosophy on waveform feedback](https://pulsescribe.me/)
- [TechCrunch — Best AI-powered dictation apps of 2025](https://techcrunch.com/2025/12/30/the-best-ai-powered-dictation-apps-of-2025/)
