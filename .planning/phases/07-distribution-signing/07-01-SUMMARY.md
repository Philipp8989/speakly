---
phase: 07-distribution-signing
plan: "01"
subsystem: distribution
tags: [tauri, distribution, signing, ci, github-actions, macos, windows]
dependency_graph:
  requires: [06-offline-fallback]
  provides: [release-pipeline, signed-installers]
  affects: [all-phases]
tech_stack:
  added: [github-actions, tauri-action]
  patterns: [tag-triggered-release, universal-binary-build, code-signing-via-secrets]
key_files:
  created:
    - .github/workflows/release.yml
    - .planning/phases/07-distribution-signing/07-01-PLAN.md
  modified:
    - src-tauri/tauri.conf.json
    - package.json
decisions:
  - "tauri-apps/tauri-action@v0 fuer CI-Build gewaehlt — offizielle Tauri-CI-Aktion"
  - "release.yml erstellt Draft-Releases — manuelle Veroeffentlichung verhindert unbeabsichtigte Releases"
  - "Signing-Secrets optional — unsigned Builds funktionieren ohne Secrets fuer interne Tests"
  - "Universal macOS Binary als CI-only-Step — lokaler cargo check ausreichend fuer Verifikation"
metrics:
  duration_seconds: 556
  completed_date: "2026-04-09"
  tasks_completed: 3
  files_changed: 4
---

# Phase 7 Plan 1: Distribution & Signing Setup Summary

App-Metadaten auf v1.0.0 aktualisiert und GitHub Actions CI-Pipeline fuer signierte macOS (universal) + Windows Release-Builds eingerichtet.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | App-Metadaten aktualisieren | 90c1a68 | src-tauri/tauri.conf.json, package.json |
| 2 | Rust-Targets pruefen + cargo check | (toolchain-only) | — |
| 3 | GitHub Actions release.yml | 219a3e2 | .github/workflows/release.yml |

## What Was Built

**App Metadata (Task 1):**
- `src-tauri/tauri.conf.json`: version `0.1.0` → `1.0.0`
- `package.json`: version `0.1.0` → `1.0.0`, name `tmpspeakly-scaffold` → `speakly`

**Rust Targets (Task 2):**
- `aarch64-apple-darwin` bereits installiert (native)
- `x86_64-apple-darwin` hinzugefuegt via `rustup target add`
- `cargo check` laeuft fehlerfrei (4 nicht-kritische Warnings)

**GitHub Actions Pipeline (Task 3):**
`.github/workflows/release.yml` mit zwei Jobs:
- `build-macos`: macOS universal binary (`--target universal-apple-darwin`) via `tauri-apps/tauri-action@v0`
- `build-windows`: Windows x86_64 MSI (`--target x86_64-pc-windows-msvc`)

Trigger: Tag-Push `v*` oder manuell via `workflow_dispatch`.

**Signing-Secrets (fuer echte Releases konfigurieren):**
| Secret | Plattform | Zweck |
|--------|-----------|-------|
| `APPLE_CERTIFICATE` | macOS | Base64 .p12 Developer-ID Zertifikat |
| `APPLE_CERTIFICATE_PASSWORD` | macOS | .p12 Passwort |
| `APPLE_SIGNING_IDENTITY` | macOS | "Developer ID Application: ..." |
| `APPLE_ID` | macOS | Apple-ID fuer Notarization |
| `APPLE_PASSWORD` | macOS | App-spezifisches Passwort |
| `APPLE_TEAM_ID` | macOS | 10-stelliger Team-Code |
| `WINDOWS_CERTIFICATE` | Windows | Base64 .pfx Zertifikat (optional) |
| `WINDOWS_CERTIFICATE_PASSWORD` | Windows | .pfx Passwort (optional) |
| `TAURI_SIGNING_PRIVATE_KEY` | beide | Tauri Update-Signing (optional) |

## Deviations from Plan

None — plan executed exactly as written.

## Known Notes

**Universal macOS Build ist CI-only:** `tauri build --target universal-apple-darwin` dauert 15-30 Minuten lokal. Der lokale Verifikationsschritt verwendet `cargo check` (nativ), was in ~60 Sekunden abschliesst. Der universale Build wird ausschliesslich in GitHub Actions ausgefuehrt.

**Unsigned Builds funktionieren ohne Secrets:** Die Pipeline baut auch ohne konfigurierte Signing-Secrets — die Binaries werden dann nicht signiert/notarisiert. Fuer interne Teamtests ist das ausreichend; fuer externe Distribution muessen die Apple-Secrets gesetzt werden.

## Self-Check: PASSED

| Check | Result |
|-------|--------|
| .github/workflows/release.yml | FOUND |
| src-tauri/tauri.conf.json | FOUND |
| package.json | FOUND |
| commit 90c1a68 (metadata) | FOUND |
| commit 219a3e2 (CI pipeline) | FOUND |
