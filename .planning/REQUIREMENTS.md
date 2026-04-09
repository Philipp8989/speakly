# Requirements: Speakly

**Defined:** 2026-04-09
**Core Value:** Per Tastendruck gesprochenen Text zuverlaessig und schnell in jedes Textfeld auf dem Computer einfuegen — plattformuebergreifend, kostenlos, fuer das ganze Team.

## v1 Requirements

Requirements for initial release. Each maps to roadmap phases.

### Recording & Input

- [ ] **REC-01**: User kann per globalem Hotkey Aufnahme starten/stoppen (Toggle-Modus)
- [ ] **REC-02**: User kann per Hold-to-Record (Push-to-Talk) diktieren
- [ ] **REC-03**: User sieht Waveform/Audio-Level-Feedback waehrend der Aufnahme
- [ ] **REC-04**: User kann Mikrofon in den Einstellungen auswaehlen
- [ ] **REC-05**: Aufnahme stoppt automatisch nach konfigurierbarer Stille-Dauer

### Speech-to-Text

- [ ] **STT-01**: User bekommt Transkription via OpenAI Whisper API (Cloud)
- [ ] **STT-02**: Transkription enthaelt automatische Punktuation und Gross-/Kleinschreibung
- [ ] **STT-03**: Sprache wird automatisch erkannt (50+ Sprachen)
- [ ] **STT-04**: User kann auf lokales whisper.cpp umschalten (Offline-Fallback)
- [ ] **STT-05**: User kann bevorzugte STT-Engine waehlen (Cloud/Lokal/Auto)

### Text-Einfuegen

- [ ] **INS-01**: Transkribierter Text wird automatisch ins aktive Textfeld eingefuegt
- [ ] **INS-02**: Bisheriger Clipboard-Inhalt wird vor dem Einfuegen gesichert und danach wiederhergestellt
- [ ] **INS-03**: Text-Einfuegen nutzt App-aware Cascade (Accessibility API -> Clipboard+Paste -> Keystroke)

### AI-Commands

- [ ] **AI-01**: User kann transkribierten Text per AI-Command umformulieren (formeller, kuerzer, uebersetzen etc.)
- [ ] **AI-02**: User kann aus vordefinierten Command-Presets waehlen (Email, Slack, formal, casual)
- [ ] **AI-03**: AI-Commands nutzen Claude API mit User's eigenem API-Key

### App-Shell & Settings

- [ ] **APP-01**: System Tray Icon zeigt Status (idle/recording/processing)
- [ ] **APP-02**: User kann Hotkey in den Einstellungen aendern
- [ ] **APP-03**: User kann Sprach-Praeferenz setzen oder Auto-Detect nutzen
- [ ] **APP-04**: User kann API-Keys eingeben (OpenAI fuer Whisper, Anthropic fuer Claude)
- [ ] **APP-05**: App laeuft auf macOS und Windows
- [ ] **APP-06**: App startet minimiert im System Tray mit minimalem Ressourcenverbrauch
- [ ] **APP-07**: Erster Start zeigt Onboarding (Hotkey, API-Keys, Mikrofon, Permissions)

## v2 Requirements

Deferred to future release. Tracked but not in current roadmap.

### Enhanced UX

- **UX-01**: Status-Overlay am Cursor (floating indicator)
- **UX-02**: Custom Vocabulary / Prompt-Priming fuer Fachbegriffe
- **UX-03**: Aufnahme-Historie (letzte Transkriptionen anzeigen)

### Integration

- **INT-01**: Auto-Update Mechanismus
- **INT-02**: Keyboard-Shortcut fuer AI-Commands ohne vorherige Aufnahme (Clipboard-Text umformulieren)

## Out of Scope

Explicitly excluded. Documented to prevent scope creep.

| Feature | Reason |
|---------|--------|
| Mobile App | Desktop-first, mobile spaeter |
| Browser Extension | Tauri-App deckt alles ab |
| Eigenes AI-Modell | Bestehende APIs (Whisper, Claude) reichen |
| Echtzeit-Streaming | Batch nach Stop reicht fuer v1 |
| Team-Management | Jeder installiert lokal, kein Backend |
| Monetarisierung | Internes Tool, kostenlos |
| Transkript-Historie/Suche | Ziel-App (Gmail, Docs etc.) owned die Historie |
| Voice-Commands fuer OS | Anderes Produktkategorie |
| Custom Wake Word | Hotkey ist das richtige Aktivierungsmodell |
| Eigenes Notiz-Fenster | Text geht in die fokussierte App |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| APP-01 | Phase 1 | Pending |
| APP-05 | Phase 1 | Pending |
| APP-06 | Phase 1 | Pending |
| APP-02 | Phase 2 | Pending |
| APP-04 | Phase 2 | Pending |
| APP-07 | Phase 2 | Pending |
| REC-01 | Phase 3 | Pending |
| REC-02 | Phase 3 | Pending |
| REC-03 | Phase 3 | Pending |
| REC-04 | Phase 3 | Pending |
| REC-05 | Phase 3 | Pending |
| STT-01 | Phase 4 | Pending |
| STT-02 | Phase 4 | Pending |
| STT-03 | Phase 4 | Pending |
| APP-03 | Phase 4 | Pending |
| INS-01 | Phase 4 | Pending |
| INS-02 | Phase 4 | Pending |
| INS-03 | Phase 4 | Pending |
| AI-01 | Phase 5 | Pending |
| AI-02 | Phase 5 | Pending |
| AI-03 | Phase 5 | Pending |
| STT-04 | Phase 6 | Pending |
| STT-05 | Phase 6 | Pending |

**Coverage:**
- v1 requirements: 22 total
- Mapped to phases: 22
- Unmapped: 0

---
*Requirements defined: 2026-04-09*
*Last updated: 2026-04-09 after roadmap creation*
