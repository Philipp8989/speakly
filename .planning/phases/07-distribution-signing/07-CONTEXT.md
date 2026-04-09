# Phase 7: Distribution & Signing - Context

**Gathered:** 2026-04-09
**Status:** Ready for planning

<domain>
## Phase Boundary

Create signed, distributable installers for macOS (.dmg) and Windows (.msi/.exe) that team members can install without security warnings. Set up GitHub Actions CI to build and sign both platforms on tag push.

</domain>

<decisions>
## Implementation Decisions

### macOS Distribution
- **D-01:** Build universal macOS binary (arm64 + x86_64) via `tauri build --target universal-apple-darwin`.
- **D-02:** Output format: .dmg installer. Tauri generates this by default on macOS.
- **D-03:** Code signing with Apple Developer certificate. Use `APPLE_SIGNING_IDENTITY` env var.
- **D-04:** Notarization via `notarytool` — required for Gatekeeper to allow installation without warning.
- **D-05:** Notarization needs Apple ID, team ID, and app-specific password as CI secrets.

### Windows Distribution
- **D-06:** Output format: .msi installer (Tauri default via WiX on Windows) OR .exe via NSIS.
- **D-07:** Code signing with a code signing certificate. Use `TAURI_SIGNING_PRIVATE_KEY` env var.
- **D-08:** Without signing, SmartScreen will warn — acceptable for internal team use but signing preferred.

### GitHub Actions CI
- **D-09:** Single workflow file `.github/workflows/release.yml` triggered on tag push (e.g., `v1.0.0`).
- **D-10:** Matrix build: macOS (macos-latest) + Windows (windows-latest).
- **D-11:** Steps: checkout → setup Rust → setup Node + pnpm → install deps → build → sign → upload as GitHub Release assets.
- **D-12:** Secrets stored in GitHub repo settings: `APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`, `APPLE_ID`, `APPLE_TEAM_ID`, `APPLE_PASSWORD`, `TAURI_SIGNING_PRIVATE_KEY`.

### Tauri Configuration
- **D-13:** Update `tauri.conf.json` with proper app metadata: identifier `de.speakly.app`, version from `package.json`.
- **D-14:** App icons already exist from scaffold (Phase 1). Verify they look correct.

### Claude's Discretion
- Exact CI workflow syntax and caching strategy
- Whether to include auto-update mechanism (Tauri updater plugin) — can defer
- DMG background image and window layout

</decisions>

<canonical_refs>
## Canonical References

### Existing Code
- `src-tauri/tauri.conf.json` — Current config, needs metadata update
- `package.json` — Version field
- `.github/` — Does not exist yet, create workflows dir

### Research
- `.planning/research/PITFALLS.md` — Windows SmartScreen, macOS notarization requirements

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- Tauri build system already configured from Phase 1 scaffold
- App icons exist in `src-tauri/icons/`

### Integration Points
- `tauri.conf.json` needs identifier + version update
- `.github/workflows/release.yml` new file
- Secrets configured manually in GitHub repo settings (not automated)

</code_context>

<specifics>
## Specific Ideas

- Team members should be able to download from GitHub Releases and install with one click
- No auto-update in v1 — manual download of new versions is fine

</specifics>

<deferred>
## Deferred Ideas

- Auto-update via Tauri updater plugin — v2
- Linux support — not needed for team (all Mac/Windows)
- Homebrew/winget package — overkill for internal tool

</deferred>

---

*Phase: 07-distribution-signing*
*Context gathered: 2026-04-09*
