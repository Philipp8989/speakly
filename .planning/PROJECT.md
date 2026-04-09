# Speakly

## What This Is

Cross-Platform Voice-to-Text Desktop App fuer Mac und Windows. Open-Source Alternative zu Voicy. Globaler Hotkey starten, sprechen, Text wird automatisch ins aktive Textfeld eingefuegt — egal ob E-Mail, Slack, Docs oder Code-Editor. Mit AI-Commands zum Umformulieren von diktiertem Text.

## Core Value

Per Tastendruck gesprochenen Text zuverlaessig und schnell in jedes Textfeld auf dem Computer einfuegen — plattformuebergreifend, kostenlos, fuer das ganze Team.

## Requirements

### Validated

(None yet — ship to validate)

### Active

- [ ] Globaler Hotkey zum Starten/Stoppen der Aufnahme
- [ ] Audio-Aufnahme ueber System-Mikrofon
- [ ] Speech-to-Text via OpenAI Whisper API (Cloud, primaere Engine)
- [ ] Speech-to-Text via lokales Whisper-Modell (Offline-Fallback)
- [ ] Automatische Spracherkennung (50+ Sprachen)
- [ ] Automatische Punktuation und Gross-/Kleinschreibung
- [ ] Text in aktives Textfeld einfuegen (Clipboard + Paste-Simulation)
- [ ] AI-Commands zum Umformulieren (formeller, kuerzer, uebersetzen etc.)
- [ ] System-Tray Icon mit Status-Anzeige
- [ ] Einstellungen: Hotkey, Sprache, Cloud/Lokal, API-Keys
- [ ] Cross-Platform: macOS und Windows Support
- [ ] Minimaler Ressourcenverbrauch im Hintergrund

### Out of Scope

- Mobile App — Desktop-first, mobile spaeter
- Browser Extension — Tauri-App deckt alles ab
- Eigenes AI-Modell trainieren — nutzen bestehende APIs (Whisper, Claude)
- Echtzeit-Streaming-Transkription — Batch nach Aufnahme-Stopp reicht fuer v1
- Multi-User/Team-Management — jeder installiert lokal, kein Backend noetig
- Bezahlmodell/Monetarisierung — internes Tool, kostenlos

## Context

- Team arbeitet auf Mac und Windows, braucht einheitliche Loesung
- Voicy Pro kostet $8.49/Monat pro User — unnoetige Kosten
- Windows-User im Team koennen Voicy nicht nutzen
- Tauri gewaehlt wegen kleiner Bundle-Groesse und nativer Performance
- OpenAI Whisper API fuer beste Qualitaet, lokales Whisper als Offline-Fallback
- Claude API fuer AI-Commands (Umformulierung, Uebersetzung)
- Nutzer brauchen es zum Diktieren (E-Mails, Docs, Slack) und fuer schnelle Notizen

## Constraints

- **Tech Stack**: Tauri (Rust + Web-Frontend) — cross-platform, klein, schnell
- **STT**: OpenAI Whisper API (Cloud) + whisper.cpp (lokal) — beste Kombination aus Qualitaet und Offline-Faehigkeit
- **AI**: Claude API fuer Text-Umformulierung — bereits im Team genutzt
- **Plattform**: Muss auf macOS UND Windows funktionieren — keine plattformspezifischen Hacks
- **Budget**: Keine laufenden Kosten ausser API-Usage (Whisper + Claude) — User bringen eigene API-Keys mit

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Tauri statt Electron | Kleinere Bundle-Groesse, weniger RAM, native Performance | -- Pending |
| Whisper API + lokales Fallback | Beste Qualitaet online, Offline-Faehigkeit wenn noetig | -- Pending |
| Claude API fuer AI-Commands | Team nutzt bereits Claude, konsistente API | -- Pending |
| Clipboard + Paste statt OS-Input | Zuverlaessigster Weg Text in beliebige Apps einzufuegen | -- Pending |
| User bringt eigene API-Keys | Kein Backend noetig, keine laufenden Kosten | -- Pending |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd-transition`):
1. Requirements invalidated? -> Move to Out of Scope with reason
2. Requirements validated? -> Move to Validated with phase reference
3. New requirements emerged? -> Add to Active
4. Decisions to log? -> Add to Key Decisions
5. "What This Is" still accurate? -> Update if drifted

**After each milestone** (via `/gsd-complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-04-09 after initialization*
