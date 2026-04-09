---
phase: 1
slug: app-shell
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-09
---

# Phase 1 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | None established yet — greenfield project |
| **Config file** | None — Wave 0 creates scaffold |
| **Quick run command** | `pnpm tauri dev` (manual verification) |
| **Full suite command** | N/A — Phase 1 is UI/shell; automated tests deferred to Phase 3+ |
| **Estimated runtime** | ~30 seconds (build + manual check) |

---

## Sampling Rate

- **After every task commit:** Run `pnpm tauri dev` and visually verify
- **After every plan completes:** Full manual smoke test of all success criteria

---

## Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| APP-01 | Tray icon visible after launch | Manual smoke | `pnpm tauri dev` → visually inspect tray | No — manual |
| APP-01 | Clicking tray opens popup window | Manual smoke | `pnpm tauri dev` → click tray icon | No — manual |
| APP-05 | App builds and runs on macOS | Manual smoke | `pnpm tauri build` (macOS runner) | No — manual |
| APP-05 | App builds and runs on Windows | Manual smoke | `pnpm tauri build` (Windows runner) | No — CI only |
| APP-06 | App starts in tray, no window shown | Manual smoke | `pnpm tauri dev` → check no window spawns | No — manual |
| APP-06 | Closing window hides to tray, not quit | Manual smoke | Open popup → click X → verify tray still present | No — manual |
| ConfigStore | Test key persists across restarts | Manual smoke | Restart app → check store file contains key | No — manual |

---

## Wave 0 Gaps

- [ ] No test files exist — project not yet scaffolded
- [ ] Rust toolchain not installed — install before any work begins
- [ ] `src-tauri/capabilities/default.json` — needs plugin permissions added during scaffold

---

*Phase: 01-app-shell*
*Validation strategy created: 2026-04-09*
