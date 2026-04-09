# Roadmap: Speakly

## Overview

Seven phases deliver Speakly from bare Tauri scaffold to signed, distributable cross-platform app. Phases 1-4 form the shippable MVP: hotkey → mic → Whisper API → text in the active field. Phase 5 adds AI reformulation. Phase 6 adds offline fallback. Phase 7 seals distribution for team rollout. Each phase delivers a runnable, verifiable increment — no phase leaves the app in a broken state.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [x] **Phase 1: App Shell** - Tauri scaffold + system tray + cross-platform lifecycle (no audio) (completed 2026-04-09)
- [x] **Phase 2: Settings & Onboarding** - Hotkey config, API key input, permission flow, first-run onboarding (completed 2026-04-09)
- [x] **Phase 3: Recording Core** - Global hotkey + audio capture + waveform feedback + VAD gate (completed 2026-04-09)
- [x] **Phase 4: Cloud STT + Text Injection (MVP)** - Whisper API + clipboard-safe paste into active field (completed 2026-04-09)
- [ ] **Phase 5: AI Commands** - Claude API reformulation with preset commands
- [ ] **Phase 6: Offline Fallback** - whisper.cpp local STT + engine selection
- [ ] **Phase 7: Distribution & Signing** - macOS notarization + Windows signing + CI pipeline

## Phase Details

### Phase 1: App Shell
**Goal**: A runnable Tauri app lives in the system tray on macOS and Windows with correct lifecycle behavior
**Depends on**: Nothing (first phase)
**Requirements**: APP-01, APP-05, APP-06
**Success Criteria** (what must be TRUE):
  1. App launches and appears as a tray icon (idle state) on macOS and Windows
  2. Clicking the tray icon opens a minimal popup/window
  3. Closing the window hides to tray instead of quitting on Windows
  4. App consumes minimal resources when idle (no audio, no network)
  5. ConfigStore is initialized and persists a test key across restarts
**Plans**: 4 plans
Plans:
- [x] 01-01-PLAN.md — Scaffold Tauri project, install plugins (store, positioner, global-shortcut), configure capabilities and window
- [x] 01-02-PLAN.md — Rust tray backend: TrayIconBuilder, positioned popup toggle, quit menu, close-to-tray intercept
- [x] 01-03-PLAN.md — React frontend: TrayPopup UI, ConfigStore init, global hotkey registration stub
- [x] 01-04-PLAN.md — Human verification of all 5 Phase 1 success criteria
**UI hint**: yes

### Phase 2: Settings & Onboarding
**Goal**: User can configure all required settings and complete first-run onboarding before recording begins
**Depends on**: Phase 1
**Requirements**: APP-02, APP-04, APP-07
**Success Criteria** (what must be TRUE):
  1. First launch shows onboarding screen covering hotkey, API keys, microphone, and required permissions
  2. User can enter and save OpenAI API key and Anthropic API key
  3. User can remap the global hotkey from the settings UI
  4. macOS Microphone and Accessibility permissions are checked; user is deep-linked to System Preferences if denied
  5. All settings persist across app restarts
**Plans**: 4 plans
Plans:
- [x] 02-01-PLAN.md — Rust backend: reqwest + cpal deps, Tauri commands (validate keys, list mics, check permissions), ConfigStore Phase 2 defaults, updated capabilities
- [x] 02-02-PLAN.md — Settings UI: useSettings hook + SettingsPage (API keys section + hotkey remap section)
- [x] 02-03-PLAN.md — Onboarding wizard (4 steps) + App.tsx wiring (window creation, onboarding check, URL routing)
- [x] 02-04-PLAN.md — Human verification of all 5 Phase 2 success criteria
**UI hint**: yes

### Phase 3: Recording Core
**Goal**: User can trigger audio recording via hotkey and see live feedback, with safety gates preventing bad audio from reaching STT
**Depends on**: Phase 2
**Requirements**: REC-01, REC-02, REC-03, REC-04, REC-05
**Success Criteria** (what must be TRUE):
  1. Pressing the configured hotkey starts recording; pressing again stops it (toggle mode)
  2. Holding the hotkey records (push-to-talk); releasing stops recording
  3. Tray icon changes state during recording and processing
  4. Audio level / waveform is visible in the popup while recording
  5. Recording stops automatically after the configured silence duration; sub-0.5s or silent recordings are discarded without pasting anything
**Plans**: 4 plans
Plans:
- [x] 03-01-PLAN.md — Rust recording module: cpal capture, RMS VAD, hound WAV encoder, silence auto-stop, unit tests
- [x] 03-02-PLAN.md — Tauri wiring: RecordingState managed state, hotkey with_handler (hold vs toggle), tray icon ID fix, ConfigStore Phase 3 defaults, JS stub removal
- [x] 03-03-PLAN.md — Frontend: AudioLevelBars component, TrayPopup extended with recording state + event listeners
- [x] 03-04-PLAN.md — Human verification of all 5 Phase 3 success criteria

### Phase 4: Cloud STT + Text Injection (MVP)
**Goal**: Dictated speech is reliably transcribed and injected into the active text field without destroying the user's clipboard
**Depends on**: Phase 3
**Requirements**: STT-01, STT-02, STT-03, APP-03, INS-01, INS-02, INS-03
**Success Criteria** (what must be TRUE):
  1. After recording stops, transcribed text appears in the currently focused text field within 2-3 seconds
  2. Transcription includes correct punctuation and capitalization (Whisper API quality)
  3. Language is auto-detected; user can also set a preferred language in settings
  4. The user's original clipboard content is intact after paste (save/restore works)
  5. End-to-end flow works in at least: browser text field, VS Code, Slack, macOS Notes
**Plans**: 3 plans
Plans:
- [x] 04-01-PLAN.md — Rust backend: add enigo + clipboard-manager deps, stt/mod.rs (Whisper API), inject/mod.rs (clipboard save/paste/restore), transcribe_and_inject command
- [x] 04-02-PLAN.md — Frontend: TrayPopup 5-state machine, App.tsx event wiring (recording_stopped + transcription_state_changed), language dropdown in SettingsPage
- [x] 04-03-PLAN.md — Human verification of all 5 Phase 4 success criteria

### Phase 5: AI Commands
**Goal**: User can apply AI reformulation commands to transcribed text before it is inserted
**Depends on**: Phase 4
**Requirements**: AI-01, AI-02, AI-03
**Success Criteria** (what must be TRUE):
  1. After transcription completes, user can choose an AI command (formal, shorter, translate, email, Slack) from the popup
  2. Applying a command sends text to Claude API and inserts the reformulated result
  3. Commands use the user's own Anthropic API key from settings
  4. User can skip AI commands and insert raw transcription directly
**Plans**: TBD
**UI hint**: yes

### Phase 6: Offline Fallback
**Goal**: User can transcribe without internet using a local whisper.cpp model, and choose which STT engine to use
**Depends on**: Phase 4
**Requirements**: STT-04, STT-05
**Success Criteria** (what must be TRUE):
  1. User can switch to local whisper.cpp engine in settings and record successfully without internet
  2. User can select STT engine preference: Cloud, Local, or Auto (local when offline)
  3. First-time local use triggers model download with visible progress
  4. Windows shows a speed warning when using local mode on non-AVX2 hardware
**Plans**: TBD

### Phase 7: Distribution & Signing
**Goal**: Signed, notarized app installers exist for macOS (arm64 + x86_64) and Windows that any team member can install without security warnings
**Depends on**: Phase 6
**Requirements**: (no new v1 requirements — enables delivery of all prior phases to team)
**Success Criteria** (what must be TRUE):
  1. macOS .dmg installs without Gatekeeper warning (notarization complete)
  2. Windows .msi installs without SmartScreen warning (signed binary)
  3. Universal macOS binary runs on both Apple Silicon and Intel Macs
  4. GitHub Actions CI builds and signs both platforms on tag push
**Plans**: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 → 3 → 4 → 5 → 6 → 7

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. App Shell | 4/4 | Complete    | 2026-04-09 |
| 2. Settings & Onboarding | 4/4 | Complete    | 2026-04-09 |
| 3. Recording Core | 4/4 | Complete    | 2026-04-09 |
| 4. Cloud STT + Text Injection | 3/3 | Complete    | 2026-04-09 |
| 5. AI Commands | 0/TBD | Not started | - |
| 6. Offline Fallback | 0/TBD | Not started | - |
| 7. Distribution & Signing | 0/TBD | Not started | - |
